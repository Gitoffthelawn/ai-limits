use chrono::{DateTime, Utc};

use crate::types::{
    AccountInfo, CliAuthorization, LimitInfo, SourceData, SourceStatus, StructuredSourceInfo,
    TokenUsage, UsageInfo,
};

use super::parse::{
    format_unix_utc, parse_balance, plan_name, remaining_percent, CodexRpcResponses,
    RateLimitWindow, ResetCreditsSummary,
};

const PROVIDER: &str = "codex";
const SOURCE: &str = "codex_rpc";
const SOURCE_LINK: &str = "https://developers.openai.com/codex/cli";

pub(super) const CLI_MISSING_MESSAGE: &str = "Codex CLI is not installed or is not available in PATH; install `codex` and try again. Setup: https://developers.openai.com/codex/cli";
pub(super) const AUTHORIZATION_MESSAGE: &str = "Codex CLI is installed but not authorized; run `codex login` and try again. Setup: https://developers.openai.com/codex/cli";
const NO_DATA_MESSAGE: &str = "no supported limit or usage data in the Codex app-server response";

const PRIMARY_LIMIT_NAME: &str = "primary";
const SECONDARY_LIMIT_NAME: &str = "secondary";
const AVAILABLE_RESET_STATUS: &str = "available";

pub(super) fn build_source_data(
    responses: &CodexRpcResponses,
    collected_at: DateTime<Utc>,
    data_as_of: DateTime<Utc>,
) -> SourceData {
    let structured = build_structured(responses, collected_at, data_as_of);
    let raw = serde_json::to_string_pretty(responses).ok();

    SourceData {
        raw,
        structured,
        stderr: String::new(),
    }
}

/// Projects the responses per the table in
/// [codex-rpc-usage.md](../../../docs/get-limits/providers/codex-rpc-usage.md).
/// Every value that cannot be read as reported degrades to `null` plus a short
/// fixed diagnostic; nothing is guessed.
fn build_structured(
    responses: &CodexRpcResponses,
    collected_at: DateTime<Utc>,
    data_as_of: DateTime<Utc>,
) -> StructuredSourceInfo {
    let mut diagnostics = Vec::new();
    let mut account = AccountInfo::default();
    let mut limits = Vec::new();
    let mut available_limit_resets = None;

    match responses.account.as_ref() {
        Some(response) => {
            let plan_type = response
                .account
                .as_ref()
                .and_then(|account| account.plan_type.as_deref());
            account.plan = plan_type.and_then(plan_name).map(str::to_string);
            if plan_type.is_some() && account.plan.is_none() {
                diagnostics.push("account plan: the reported plan tier is `unknown`".to_string());
            }
        }
        None => diagnostics.push("account: the response could not be read".to_string()),
    }

    match responses.codex_limits() {
        Some(snapshot) => {
            if let Some(primary) = snapshot.primary.as_ref() {
                limits.push(limit_from_window(
                    PRIMARY_LIMIT_NAME,
                    snapshot.limit_name.as_deref(),
                    primary,
                ));
            }
            if let Some(secondary) = snapshot.secondary.as_ref() {
                limits.push(limit_from_window(
                    SECONDARY_LIMIT_NAME,
                    snapshot.limit_name.as_deref(),
                    secondary,
                ));
            }
            account.credits_remaining = snapshot.credits.as_ref().and_then(|credits| {
                credits_remaining(
                    credits.unlimited,
                    credits.balance.as_deref(),
                    &mut diagnostics,
                )
            });
            if let Some(note) = rate_limit_reached_note(snapshot.rate_limit_reached_type.as_deref())
            {
                diagnostics.push(note.to_string());
            }
        }
        None => diagnostics.push("limits: the `codex` rate-limit entry is missing".to_string()),
    }

    if let Some(summary) = responses
        .rate_limits
        .as_ref()
        .and_then(|response| response.rate_limit_reset_credits.as_ref())
    {
        available_limit_resets = reset_count(summary, &mut diagnostics);
        if let Some(note) = reset_credits_note(summary) {
            diagnostics.push(note);
        }
    }

    let usage = match responses.usage.as_ref() {
        Some(response) => {
            let total = response
                .summary
                .as_ref()
                .and_then(|summary| summary.lifetime_tokens);
            if total.is_none() {
                diagnostics.push("usage: the lifetime token total is not reported".to_string());
            }
            UsageInfo {
                tokens: TokenUsage {
                    total,
                    ..TokenUsage::default()
                },
                ..UsageInfo::default()
            }
        }
        None => {
            diagnostics.push("usage: the response could not be read".to_string());
            UsageInfo::default()
        }
    };

    let data_available = !limits.is_empty()
        || account.plan.is_some()
        || account.credits_remaining.is_some()
        || available_limit_resets.is_some()
        || usage.tokens.total.is_some();

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available,
            access_available: true,
            message: (!data_available).then(|| NO_DATA_MESSAGE.to_string()),
            cli_authorization: None,
        },
        raw_data_available: true,
        collected_at: Some(format_utc(collected_at)),
        data_as_of: data_available.then(|| format_utc(data_as_of)),
        account,
        limits,
        available_limit_resets,
        usage,
        diagnostics,
    }
}

pub(super) fn authorization_required_source_data() -> SourceData {
    unavailable_source_data(AUTHORIZATION_MESSAGE, Some(CliAuthorization::Codex))
}

pub(super) fn unavailable_source_data(
    message: &str,
    cli_authorization: Option<CliAuthorization>,
) -> SourceData {
    SourceData {
        raw: None,
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some(message.to_string()),
                cli_authorization,
            },
            raw_data_available: false,
            collected_at: Some(format_utc(Utc::now())),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        },
        stderr: String::new(),
    }
}

/// `primary` and `secondary` produce two separate records that are told apart
/// by `window_minutes` and `resets_at`; a missing `secondary` produces no
/// record at all.
fn limit_from_window(name: &str, limit_name: Option<&str>, window: &RateLimitWindow) -> LimitInfo {
    LimitInfo {
        name: limit_name.unwrap_or(name).to_string(),
        window_label: limit_name.map(str::to_string),
        window_minutes: window.window_duration_mins,
        resets_at: window.resets_at.and_then(format_unix_utc),
        used_percent: window.used_percent,
        remaining_percent: window.used_percent.map(remaining_percent),
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    }
}

/// An unlimited balance is not a remaining amount, and an unparseable one is
/// never emitted as a number-shaped guess.
fn credits_remaining(
    unlimited: bool,
    balance: Option<&str>,
    diagnostics: &mut Vec<String>,
) -> Option<f64> {
    if unlimited {
        diagnostics
            .push("credits: the balance is unlimited and is not a remaining amount".to_string());
        return None;
    }

    let balance = balance?;
    let parsed = parse_balance(balance);
    if parsed.is_none() {
        diagnostics.push("credits: the reported balance could not be read as a number".to_string());
    }
    parsed
}

/// The count is taken as reported and is never recomputed from the records.
fn reset_count(summary: &ResetCreditsSummary, diagnostics: &mut Vec<String>) -> Option<u64> {
    let count = summary.available_count?;
    match u64::try_from(count) {
        Ok(count) => Some(count),
        Err(_) => {
            diagnostics.push("limit resets: the reported count could not be read".to_string());
            None
        }
    }
}

/// Expiry and reset type come from the reset-credit records, which inform a
/// diagnostic and nothing else.
fn reset_credits_note(summary: &ResetCreditsSummary) -> Option<String> {
    let credits = summary.credits.as_ref()?;
    let expires_at = credits
        .iter()
        .filter(|credit| credit.status.as_deref() == Some(AVAILABLE_RESET_STATUS))
        .filter_map(|credit| credit.expires_at)
        .min()?;

    format_unix_utc(expires_at)
        .map(|value| format!("limit resets: the earliest one expires {value}"))
}

/// The reached-type enum informs a diagnostic and nothing else. Values are
/// mapped to fixed literals so that no response content is interpolated.
fn rate_limit_reached_note(reached_type: Option<&str>) -> Option<&'static str> {
    match reached_type? {
        "rate_limit_reached" => Some("limits: the account rate limit has been reached"),
        "workspace_owner_credits_depleted" | "workspace_member_credits_depleted" => {
            Some("limits: the workspace credits are depleted")
        }
        "workspace_owner_usage_limit_reached" | "workspace_member_usage_limit_reached" => {
            Some("limits: the workspace usage limit has been reached")
        }
        _ => Some("limits: the reported rate-limit state is not recognized"),
    }
}

fn format_utc(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
