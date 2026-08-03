use std::io::{self, Write};
use std::process::Stdio;

use crate::infra::os_access::{cursor_dashboard_request_command, read_cursor_access_token};

use super::parse::{
    billing_window, is_enterprise_account, monthly_billing_window, plan_usage_is_present, team_id,
    UsageEventPages, PAGE_SIZE,
};

/// Hard ceiling on `GetFilteredUsageEvents` pages. Reaching it makes the page
/// set incomplete, which turns the session count into `null` rather than into
/// an understated number.
const MAX_EVENT_PAGES: u64 = 60;

pub(super) struct AccessDenied {
    pub message: String,
}

/// The body of one dashboard response, or a fixed diagnostic literal.
///
/// The error side is always a `&'static str`: no response body, HTTP status, or
/// `curl` message is ever carried out of this module.
pub type MethodOutcome = Result<String, &'static str>;

/// Every response the source collected in one run.
#[derive(Clone, Debug, Default)]
pub struct CursorResponses {
    pub plan_info: Option<MethodOutcome>,
    pub current_period: Option<MethodOutcome>,
    pub hard_limit: Option<MethodOutcome>,
    pub aggregated: Option<MethodOutcome>,
    pub filtered: UsageEventPages,
    /// Diagnostics produced while fetching. Fixed literals only.
    pub fetch_diagnostics: Vec<&'static str>,
}

const TOKEN_MISSING: &str =
    "Cursor api2 usage unavailable: token not found; run `cursor agent login`";
const TOKEN_EMPTY: &str = "Cursor api2 usage unavailable: empty token; run `cursor agent login`";
const TOKEN_UNREADABLE: &str =
    "Cursor api2 usage unavailable: the macOS Keychain token could not be read";
const TOKEN_REJECTED: &str =
    "Cursor api2 usage unavailable: token rejected; run `cursor agent login`";
const REQUEST_FAILED: &str = "Cursor api2 usage unavailable: the request could not be completed";

const REQUEST_ERROR: &str = "request failed";
const EMPTY_RESPONSE: &str = "empty response";
const REJECTED: &str = "request was not authorized";

pub(super) fn fetch_dashboard() -> io::Result<Result<CursorResponses, AccessDenied>> {
    let token = match read_token() {
        Ok(token) => token,
        Err(denied) => return Ok(Err(denied)),
    };

    let mut responses = CursorResponses::default();

    let plan_info = post("GetPlanInfo", "{}", &token)?;
    if let Err(error) = &plan_info {
        let message = if *error == REJECTED {
            TOKEN_REJECTED
        } else {
            REQUEST_FAILED
        };
        return Ok(Err(AccessDenied {
            message: message.to_string(),
        }));
    }
    responses.plan_info = Some(plan_info);

    let current_period = post("GetCurrentPeriodUsage", "{}", &token)?;
    responses.current_period = Some(current_period);
    responses.hard_limit = Some(post("GetHardLimit", "{}", &token)?);

    let current_period_body = responses
        .current_period
        .as_ref()
        .and_then(|outcome| outcome.as_ref().ok())
        .cloned();

    let (window, team) = match current_period_body.as_deref() {
        Some(body) if plan_usage_is_present(body) => (billing_window(body), 0),
        Some(_) | None => team_window(&token, &mut responses)?,
    };

    let Some((start, end)) = window else {
        responses
            .fetch_diagnostics
            .push("usage totals and activity: the billing cycle window is unknown");
        return Ok(Ok(responses));
    };

    let window_body = format!("{{\"teamId\":{team},\"startDate\":{start},\"endDate\":{end}}}");
    responses.aggregated = Some(post("GetAggregatedUsageEvents", &window_body, &token)?);

    fetch_event_pages(&token, team, start, end, &mut responses.filtered)?;

    Ok(Ok(responses))
}

/// Team and enterprise accounts report no `planUsage`, so the window comes from
/// `GetMonthlyBillingCycle` and the aggregation is scoped to the real team.
///
/// This branch is derived from the official client only and has never run
/// against a live team account, so every step that does not answer degrades to
/// no window at all rather than to a guessed one.
fn team_window(
    token: &str,
    responses: &mut CursorResponses,
) -> io::Result<(Option<(i64, i64)>, i64)> {
    responses
        .fetch_diagnostics
        .push("team or enterprise account: the team branch is unverified against a live account");

    let me = post("GetMe", "{}", token)?;
    let Ok(me) = me else {
        responses
            .fetch_diagnostics
            .push("team account: the account response could not be read");
        return Ok((None, 0));
    };

    if is_enterprise_account(&me) != Some(true) && team_id(&me).is_none() {
        responses
            .fetch_diagnostics
            .push("team account: no team identity was reported");
        return Ok((None, 0));
    }

    let team = team_id(&me).unwrap_or(0);
    let cycle = post("GetMonthlyBillingCycle", "{}", token)?;
    let window = cycle
        .as_ref()
        .ok()
        .and_then(|body| monthly_billing_window(body));

    if window.is_none() {
        responses
            .fetch_diagnostics
            .push("team account: the monthly billing cycle window could not be read");
    }

    Ok((window, team))
}

fn fetch_event_pages(
    token: &str,
    team: i64,
    start: i64,
    end: i64,
    pages: &mut UsageEventPages,
) -> io::Result<()> {
    for page in 1..=MAX_EVENT_PAGES {
        let body = format!(
            "{{\"teamId\":{team},\"startDate\":{start},\"endDate\":{end},\"page\":{page},\"pageSize\":{PAGE_SIZE}}}"
        );
        match post("GetFilteredUsageEvents", &body, token)? {
            Ok(response) => {
                if !pages.add_page(&response) {
                    return Ok(());
                }
            }
            Err(_) => {
                pages.mark_failed();
                return Ok(());
            }
        }

        if pages.is_complete() {
            return Ok(());
        }
    }

    pages.mark_capped();
    Ok(())
}

fn read_token() -> Result<String, AccessDenied> {
    let output = read_cursor_access_token().map_err(|_| AccessDenied {
        message: TOKEN_UNREADABLE.to_string(),
    })?;

    if !output.status.success() {
        return Err(AccessDenied {
            message: TOKEN_MISSING.to_string(),
        });
    }

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        return Err(AccessDenied {
            message: TOKEN_EMPTY.to_string(),
        });
    }

    Ok(token)
}

/// Sends one Connect-RPC call. The token goes to `curl` through the config on
/// stdin, never through `argv`.
fn post(method: &str, body: &str, token: &str) -> io::Result<MethodOutcome> {
    let Some(mut command) = cursor_dashboard_request_command(method, body) else {
        return Ok(Err(REQUEST_ERROR));
    };

    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(_) => return Ok(Err(REQUEST_ERROR)),
    };

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(
            format!(
                "header = \"Authorization: Bearer {token}\"\nheader = \"Content-Type: application/json\"\nheader = \"Connect-Protocol-Version: 1\"\n"
            )
            .as_bytes(),
        )?;
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(_) => return Ok(Err(REQUEST_ERROR)),
    };

    let response = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        return Ok(Err(REQUEST_ERROR));
    }

    if response.trim().is_empty() {
        return Ok(Err(EMPTY_RESPONSE));
    }

    if is_rejected(&response) {
        return Ok(Err(REJECTED));
    }

    Ok(Ok(response))
}

fn is_rejected(response: &str) -> bool {
    response.contains("\"code\":\"unauthenticated\"")
        || response.contains("\"code\":\"permission_denied\"")
        || response.contains("\"error\":\"unauthorized\"")
        || response.contains("Unauthorized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_denied_messages_are_fixed_literals() {
        for message in [
            TOKEN_MISSING,
            TOKEN_EMPTY,
            TOKEN_UNREADABLE,
            TOKEN_REJECTED,
            REQUEST_FAILED,
        ] {
            assert!(message.starts_with("Cursor api2 usage unavailable: "));
        }
    }

    #[test]
    fn rejection_is_detected_without_carrying_the_response() {
        assert!(is_rejected(
            r#"{"code":"unauthenticated","message":"nope"}"#
        ));
        assert!(!is_rejected(r#"{"planInfo":{"planName":"Pro"}}"#));
    }
}
