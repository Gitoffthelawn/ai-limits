use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The limit bucket the product reports; the protocol keys buckets by metered
/// limit id.
pub(super) const CODEX_LIMIT_ID: &str = "codex";
const UNKNOWN_PLAN_TYPE: &str = "unknown";

/// The three responses after parsing.
///
/// Each response is parsed on its own, so a protocol change in one of them
/// degrades that part to `None` instead of dropping the whole collection.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CodexRpcResponses {
    pub(super) account: Option<AccountResponse>,
    pub(super) rate_limits: Option<RateLimitsResponse>,
    pub(super) usage: Option<UsageResponse>,
}

/// `account/read`.
///
/// `account.email` is deliberately absent from this DTO: the account email is
/// never read into the internal model, so it cannot reach structured data, raw
/// data, diagnostics, or any message.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AccountResponse {
    #[serde(default)]
    pub(super) account: Option<Account>,
    #[serde(default)]
    pub(super) requires_openai_auth: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Account {
    #[serde(default, rename = "type")]
    pub(super) account_type: Option<String>,
    #[serde(default)]
    pub(super) plan_type: Option<String>,
}

/// `account/rateLimits/read`.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RateLimitsResponse {
    /// The backward-compatible single-bucket view. It is kept for raw data
    /// only; the source reads the multi-bucket view below.
    #[serde(default)]
    pub(super) rate_limits: Option<RateLimitSnapshot>,
    #[serde(default)]
    pub(super) rate_limits_by_limit_id: Option<BTreeMap<String, RateLimitSnapshot>>,
    #[serde(default)]
    pub(super) rate_limit_reset_credits: Option<ResetCreditsSummary>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RateLimitSnapshot {
    #[serde(default)]
    pub(super) limit_id: Option<String>,
    #[serde(default)]
    pub(super) limit_name: Option<String>,
    #[serde(default)]
    pub(super) plan_type: Option<String>,
    #[serde(default)]
    pub(super) primary: Option<RateLimitWindow>,
    #[serde(default)]
    pub(super) secondary: Option<RateLimitWindow>,
    #[serde(default)]
    pub(super) credits: Option<CreditsSnapshot>,
    #[serde(default)]
    pub(super) individual_limit: Option<SpendControlLimit>,
    #[serde(default)]
    pub(super) rate_limit_reached_type: Option<String>,
}

/// A spend-control limit. It has no field in the structured schema and is kept
/// for raw data only.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SpendControlLimit {
    #[serde(default)]
    pub(super) limit: Option<String>,
    #[serde(default)]
    pub(super) used: Option<String>,
    #[serde(default)]
    pub(super) remaining_percent: Option<f64>,
    #[serde(default)]
    pub(super) resets_at: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RateLimitWindow {
    #[serde(default)]
    pub(super) used_percent: Option<f64>,
    #[serde(default)]
    pub(super) window_duration_mins: Option<u64>,
    #[serde(default)]
    pub(super) resets_at: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreditsSnapshot {
    #[serde(default)]
    pub(super) has_credits: bool,
    #[serde(default)]
    pub(super) unlimited: bool,
    #[serde(default)]
    pub(super) balance: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ResetCreditsSummary {
    #[serde(default)]
    pub(super) available_count: Option<i64>,
    #[serde(default)]
    pub(super) credits: Option<Vec<ResetCredit>>,
}

/// A manually redeemable limit reset.
///
/// The record's opaque backend `id`, `title`, and `description` are account
/// identifiers or backend copy and are deliberately not read; only the reset
/// type, status, and the two timestamps are.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ResetCredit {
    #[serde(default)]
    pub(super) reset_type: Option<String>,
    #[serde(default)]
    pub(super) status: Option<String>,
    #[serde(default)]
    pub(super) granted_at: Option<i64>,
    #[serde(default)]
    pub(super) expires_at: Option<i64>,
}

/// `account/usage/read`.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UsageResponse {
    #[serde(default)]
    pub(super) summary: Option<UsageSummary>,
    #[serde(default)]
    pub(super) daily_usage_buckets: Option<Vec<DailyUsageBucket>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UsageSummary {
    #[serde(default)]
    pub(super) lifetime_tokens: Option<u64>,
    #[serde(default)]
    pub(super) peak_daily_tokens: Option<u64>,
    #[serde(default)]
    pub(super) longest_running_turn_sec: Option<u64>,
    #[serde(default)]
    pub(super) current_streak_days: Option<u64>,
    #[serde(default)]
    pub(super) longest_streak_days: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DailyUsageBucket {
    #[serde(default)]
    pub(super) start_date: Option<String>,
    #[serde(default)]
    pub(super) tokens: Option<u64>,
}

pub(super) fn parse_responses(
    account: &Value,
    rate_limits: &Value,
    usage: &Value,
) -> CodexRpcResponses {
    CodexRpcResponses {
        account: serde_json::from_value(account.clone()).ok(),
        rate_limits: serde_json::from_value(rate_limits.clone()).ok(),
        usage: serde_json::from_value(usage.clone()).ok(),
    }
}

impl CodexRpcResponses {
    pub(super) fn codex_limits(&self) -> Option<&RateLimitSnapshot> {
        self.rate_limits
            .as_ref()?
            .rate_limits_by_limit_id
            .as_ref()?
            .get(CODEX_LIMIT_ID)
    }

    /// The account cannot be read and the CLI states that an OpenAI login is
    /// required.
    pub(super) fn requires_authorization(&self) -> bool {
        self.account
            .as_ref()
            .is_some_and(|response| response.account.is_none() && response.requires_openai_auth)
    }
}

/// The plan tier as reported. The enum value `unknown` is not a plan name.
pub(super) fn plan_name(plan_type: &str) -> Option<&str> {
    (plan_type != UNKNOWN_PLAN_TYPE).then_some(plan_type)
}

/// The balance is a high-precision decimal string; an unparseable one is never
/// emitted as a number-shaped guess.
pub(super) fn parse_balance(balance: &str) -> Option<f64> {
    balance.trim().parse().ok()
}

/// Server-reported unix seconds normalized to ISO 8601 UTC. No date
/// reconstruction and no timezone guessing are involved.
pub(super) fn format_unix_utc(seconds: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(seconds, 0)
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

pub(super) fn remaining_percent(used_percent: f64) -> f64 {
    let remaining = (100.0 - used_percent).max(0.0);
    (remaining * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_the_verified_rate_limits_payload() {
        let responses = parse_responses(
            &json!({}),
            &json!({
                "rateLimitsByLimitId": {
                    "codex": {
                        "limitId": "codex",
                        "limitName": null,
                        "primary": { "usedPercent": 100, "windowDurationMins": 10080, "resetsAt": 1786189460 },
                        "secondary": null,
                        "credits": { "hasCredits": true, "unlimited": false, "balance": "23.7854265000" },
                        "planType": "plus"
                    }
                },
                "rateLimitResetCredits": { "availableCount": 1 }
            }),
            &json!({}),
        );

        let codex = responses.codex_limits().expect("codex limit entry");
        let primary = codex.primary.as_ref().expect("primary window");
        assert_eq!(primary.used_percent, Some(100.0));
        assert_eq!(primary.window_duration_mins, Some(10080));
        assert_eq!(primary.resets_at, Some(1786189460));
        assert!(codex.secondary.is_none());
        assert_eq!(
            codex.credits.as_ref().and_then(|it| it.balance.as_deref()),
            Some("23.7854265000")
        );
    }

    #[test]
    fn each_response_degrades_on_its_own() {
        let responses = parse_responses(
            &json!({ "requiresOpenaiAuth": true }),
            &json!("unexpected"),
            &json!({ "summary": { "lifetimeTokens": 12 } }),
        );

        assert!(responses.account.is_some());
        assert!(responses.rate_limits.is_none());
        assert_eq!(
            responses
                .usage
                .as_ref()
                .and_then(|usage| usage.summary.as_ref())
                .and_then(|summary| summary.lifetime_tokens),
            Some(12)
        );
    }

    #[test]
    fn authorization_is_required_only_when_the_account_cannot_be_read() {
        let readable = parse_responses(
            &json!({ "account": { "type": "chatgpt", "planType": "plus" }, "requiresOpenaiAuth": true }),
            &json!({}),
            &json!({}),
        );
        let unreadable = parse_responses(
            &json!({ "account": null, "requiresOpenaiAuth": true }),
            &json!({}),
            &json!({}),
        );

        assert!(!readable.requires_authorization());
        assert!(unreadable.requires_authorization());
    }

    #[test]
    fn unknown_plan_type_is_not_a_plan_name() {
        assert_eq!(plan_name("plus"), Some("plus"));
        assert_eq!(plan_name("unknown"), None);
    }

    #[test]
    fn balance_parses_only_as_a_number() {
        assert_eq!(parse_balance("23.7854265000"), Some(23.7854265));
        assert_eq!(parse_balance("unlimited"), None);
        assert_eq!(parse_balance(""), None);
    }

    #[test]
    fn reset_times_are_normalized_to_iso_utc() {
        assert_eq!(
            format_unix_utc(1786189460).as_deref(),
            Some("2026-08-08T11:44:20Z")
        );
        assert_eq!(format_unix_utc(i64::MAX), None);
    }

    #[test]
    fn remaining_percent_is_derived_from_used_percent() {
        assert_eq!(remaining_percent(100.0), 0.0);
        assert_eq!(remaining_percent(16.0), 84.0);
        assert_eq!(remaining_percent(120.0), 0.0);
    }
}
