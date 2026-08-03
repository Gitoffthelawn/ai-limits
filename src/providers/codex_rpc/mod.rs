mod parse;
mod process;
mod project;

use std::io;

use chrono::Utc;

use crate::infra::os_access::{allowed_cli_command_is_available, CODEX_CLI_COMMAND};
use crate::types::SourceData;

use parse::parse_responses;
use process::read_account_session;
use project::{
    authorization_required_source_data, build_source_data, unavailable_source_data,
    CLI_MISSING_MESSAGE,
};

pub fn collect_usage() -> io::Result<SourceData> {
    if !allowed_cli_command_is_available(CODEX_CLI_COMMAND) {
        return Ok(unavailable_source_data(CLI_MISSING_MESSAGE, None));
    }

    let collected_at = Utc::now();
    let session = match read_account_session() {
        Ok(session) => session,
        Err(failure) => return Ok(unavailable_source_data(failure.message(), None)),
    };
    let data_as_of = Utc::now();

    let responses = parse_responses(&session.account, &session.rate_limits, &session.usage);
    if responses.requires_authorization() {
        return Ok(authorization_required_source_data());
    }

    Ok(build_source_data(&responses, collected_at, data_as_of))
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use serde_json::{json, Value};

    use super::parse::parse_responses;
    use super::project::{
        authorization_required_source_data, build_source_data, unavailable_source_data,
        AUTHORIZATION_MESSAGE, CLI_MISSING_MESSAGE,
    };
    use crate::types::{CliAuthorization, SourceData};

    /// A recognizable account email and the identifiers that travel with it in
    /// the verified payloads.
    const EMAIL_MARKER: &str = "account-marker@example-secret.test";
    const CREDIT_ID_MARKER: &str = "rlrc_0123456789abcdef";
    const INSTALLATION_ID_MARKER: &str = "664d2468-c822-4cf2-bc16-d55a7cb20fff";

    fn timestamp(value: &str) -> DateTime<chrono::Utc> {
        value.parse().expect("valid timestamp")
    }

    fn collected_at() -> DateTime<chrono::Utc> {
        timestamp("2026-08-03T12:00:00Z")
    }

    fn data_as_of() -> DateTime<chrono::Utc> {
        timestamp("2026-08-03T12:00:02Z")
    }

    fn account_response() -> Value {
        json!({
            "account": { "type": "chatgpt", "email": EMAIL_MARKER, "planType": "plus" },
            "requiresOpenaiAuth": true
        })
    }

    fn rate_limits_response() -> Value {
        json!({
            "rateLimits": {
                "limitId": "codex",
                "primary": { "usedPercent": 100, "windowDurationMins": 10080, "resetsAt": 1786189460 }
            },
            "rateLimitsByLimitId": {
                "codex": {
                    "limitId": "codex",
                    "limitName": null,
                    "planType": "plus",
                    "primary": { "usedPercent": 100, "windowDurationMins": 10080, "resetsAt": 1786189460 },
                    "secondary": { "usedPercent": 40, "windowDurationMins": 300, "resetsAt": 1785000000 },
                    "credits": { "hasCredits": true, "unlimited": false, "balance": "23.7854265000" },
                    "individualLimit": null,
                    "rateLimitReachedType": null
                }
            },
            "rateLimitResetCredits": {
                "availableCount": 1,
                "credits": [{
                    "id": CREDIT_ID_MARKER,
                    "resetType": "codexRateLimits",
                    "status": "available",
                    "grantedAt": 1783966467,
                    "expiresAt": 1786558467,
                    "title": "Full reset",
                    "description": "Granted to account-marker@example-secret.test"
                }]
            }
        })
    }

    fn usage_response() -> Value {
        json!({
            "summary": {
                "lifetimeTokens": 1600166325,
                "peakDailyTokens": 91327650,
                "longestRunningTurnSec": 16417,
                "currentStreakDays": 19,
                "longestStreakDays": 21
            },
            "dailyUsageBuckets": [{ "startDate": "2026-06-02", "tokens": 15074746 }]
        })
    }

    fn verified_source_data() -> SourceData {
        let responses = parse_responses(
            &account_response(),
            &rate_limits_response(),
            &usage_response(),
        );
        build_source_data(&responses, collected_at(), data_as_of())
    }

    /// Every user-visible artifact of one collection run.
    fn output_artifacts(data: &SourceData) -> Vec<String> {
        let mut artifacts = vec![
            serde_json::to_string(&data.structured).expect("structured data serializes"),
            data.stderr.clone(),
        ];
        artifacts.extend(data.raw.clone());
        artifacts.extend(data.structured.status.message.clone());
        artifacts.extend(data.structured.diagnostics.iter().cloned());
        artifacts
    }

    #[test]
    fn projects_the_verified_response_payloads() {
        let data = verified_source_data();
        let info = &data.structured;

        assert_eq!(info.provider, "codex");
        assert_eq!(info.source, "codex_rpc");
        assert!(info.status.access_available);
        assert!(info.status.data_available);
        assert!(info.raw_data_available);
        assert_eq!(info.collected_at.as_deref(), Some("2026-08-03T12:00:00Z"));
        assert_eq!(info.data_as_of.as_deref(), Some("2026-08-03T12:00:02Z"));
        assert_eq!(info.account.plan.as_deref(), Some("plus"));
        assert_eq!(info.account.credits_remaining, Some(23.7854265));
        assert_eq!(info.available_limit_resets, Some(1));
        assert_eq!(info.usage.tokens.total, Some(1600166325));
        assert_eq!(
            info.diagnostics,
            vec!["limit resets: the earliest one expires 2026-08-12T18:14:27Z".to_string()]
        );
    }

    #[test]
    fn primary_and_secondary_windows_become_two_limit_records() {
        let info = verified_source_data().structured;

        assert_eq!(info.limits.len(), 2);
        assert_eq!(info.limits[0].window_minutes, Some(10080));
        assert_eq!(info.limits[0].used_percent, Some(100.0));
        assert_eq!(info.limits[0].remaining_percent, Some(0.0));
        assert_eq!(
            info.limits[0].resets_at.as_deref(),
            Some("2026-08-08T11:44:20Z")
        );
        assert_eq!(info.limits[1].window_minutes, Some(300));
        assert_eq!(info.limits[1].remaining_percent, Some(60.0));
    }

    #[test]
    fn absolute_quota_sizes_and_subscription_fields_stay_null() {
        let info = verified_source_data().structured;

        for limit in &info.limits {
            assert!(limit.used_amount.is_none());
            assert!(limit.remaining_amount.is_none());
            assert!(limit.total_amount.is_none());
            assert!(limit.amount_unit.is_none());
        }
        assert!(info.account.credits_total.is_none());
        assert!(info.account.credits_used.is_none());
        assert!(info.account.subscription_started_at.is_none());
        assert!(info.account.renewal_at.is_none());
        assert!(info.account.price_amount.is_none());
        assert!(info.account.price_currency.is_none());
        assert!(info.account.price_period.is_none());
        assert!(info.account.price_note.is_none());
        assert!(info.account.plan_management_url.is_none());
        assert!(info.account.billing_management_url.is_none());
    }

    #[test]
    fn only_the_lifetime_total_is_reported_without_a_breakdown_or_activity() {
        let info = verified_source_data().structured;

        assert!(info.usage.tokens.input.is_none());
        assert!(info.usage.tokens.cached_input.is_none());
        assert!(info.usage.tokens.output.is_none());
        assert!(info.usage.tokens.reasoning_output.is_none());
        assert!(info.usage.tokens.cache_read.is_none());
        assert!(info.usage.tokens.cache_write.is_none());
        assert!(info.usage.activity.latest_activity_at.is_none());
        assert!(info.usage.activity.sessions_count.is_none());
    }

    #[test]
    fn a_missing_secondary_window_produces_one_record_not_a_null_one() {
        let responses = parse_responses(
            &account_response(),
            &json!({
                "rateLimitsByLimitId": {
                    "codex": {
                        "primary": { "usedPercent": 12, "windowDurationMins": 10080, "resetsAt": 1786189460 },
                        "secondary": null
                    }
                }
            }),
            &usage_response(),
        );
        let info = build_source_data(&responses, collected_at(), data_as_of()).structured;

        assert_eq!(info.limits.len(), 1);
        assert_eq!(info.limits[0].used_percent, Some(12.0));
    }

    #[test]
    fn an_unknown_plan_tier_degrades_to_null_with_a_diagnostic() {
        let responses = parse_responses(
            &json!({ "account": { "type": "chatgpt", "planType": "unknown" }, "requiresOpenaiAuth": true }),
            &rate_limits_response(),
            &usage_response(),
        );
        let info = build_source_data(&responses, collected_at(), data_as_of()).structured;

        assert!(info.account.plan.is_none());
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("plan tier is `unknown`")));
    }

    #[test]
    fn an_unlimited_balance_degrades_to_null_with_a_diagnostic() {
        let responses = parse_responses(
            &account_response(),
            &json!({
                "rateLimitsByLimitId": {
                    "codex": { "credits": { "hasCredits": true, "unlimited": true, "balance": "0" } }
                }
            }),
            &usage_response(),
        );
        let info = build_source_data(&responses, collected_at(), data_as_of()).structured;

        assert!(info.account.credits_remaining.is_none());
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("unlimited")));
    }

    #[test]
    fn an_unparseable_balance_degrades_to_null_with_a_diagnostic() {
        let responses = parse_responses(
            &account_response(),
            &json!({
                "rateLimitsByLimitId": {
                    "codex": { "credits": { "hasCredits": true, "unlimited": false, "balance": "twenty" } }
                }
            }),
            &usage_response(),
        );
        let info = build_source_data(&responses, collected_at(), data_as_of()).structured;

        assert!(info.account.credits_remaining.is_none());
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("could not be read as a number")));
    }

    #[test]
    fn a_changed_protocol_degrades_to_null_values_plus_diagnostics() {
        let responses = parse_responses(&json!("changed"), &json!("changed"), &json!("changed"));
        let data = build_source_data(&responses, collected_at(), data_as_of());
        let info = &data.structured;

        assert!(!info.status.data_available);
        assert!(info.limits.is_empty());
        assert!(info.account.plan.is_none());
        assert!(info.usage.tokens.total.is_none());
        assert!(info.data_as_of.is_none());
        assert_eq!(info.diagnostics.len(), 3);
    }

    #[test]
    fn a_missing_codex_limit_entry_degrades_to_no_limits_plus_a_diagnostic() {
        let responses = parse_responses(
            &account_response(),
            &json!({ "rateLimitsByLimitId": {} }),
            &usage_response(),
        );
        let info = build_source_data(&responses, collected_at(), data_as_of()).structured;

        assert!(info.limits.is_empty());
        assert!(info.status.data_available);
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("`codex` rate-limit entry is missing")));
    }

    #[test]
    fn an_unreadable_account_with_required_login_reports_the_authorization_state() {
        let responses = parse_responses(
            &json!({ "account": null, "requiresOpenaiAuth": true }),
            &rate_limits_response(),
            &usage_response(),
        );

        assert!(responses.requires_authorization());

        let data = authorization_required_source_data();
        assert!(!data.structured.status.access_available);
        assert!(!data.structured.status.data_available);
        assert_eq!(
            data.structured.status.cli_authorization,
            Some(CliAuthorization::Codex)
        );
        assert_eq!(
            data.structured.status.message.as_deref(),
            Some(AUTHORIZATION_MESSAGE)
        );
    }

    #[test]
    fn a_missing_cli_reports_a_fixed_setup_message() {
        let data = unavailable_source_data(CLI_MISSING_MESSAGE, None);

        assert!(!data.structured.status.access_available);
        assert!(!data.structured.raw_data_available);
        assert!(data.raw.is_none());
        assert_eq!(
            data.structured.status.message.as_deref(),
            Some(CLI_MISSING_MESSAGE)
        );
    }

    #[test]
    fn the_account_email_and_identifiers_never_reach_any_output_artifact() {
        let data = verified_source_data();
        let artifacts = output_artifacts(&data);

        assert!(data.raw.is_some(), "raw data is exposed for this source");
        for artifact in &artifacts {
            for marker in [EMAIL_MARKER, CREDIT_ID_MARKER, "example-secret.test"] {
                assert!(
                    !artifact.contains(marker),
                    "an account identifier reached an output artifact"
                );
            }
        }
    }

    #[test]
    fn identifiers_from_protocol_notifications_never_reach_any_output_artifact() {
        let responses = parse_responses(
            &json!({
                "account": { "type": "chatgpt", "email": EMAIL_MARKER, "planType": "plus" },
                "requiresOpenaiAuth": true,
                "installationId": INSTALLATION_ID_MARKER
            }),
            &rate_limits_response(),
            &usage_response(),
        );
        let data = build_source_data(&responses, collected_at(), data_as_of());

        for artifact in output_artifacts(&data) {
            assert!(!artifact.contains(INSTALLATION_ID_MARKER));
            assert!(!artifact.contains(EMAIL_MARKER));
        }
    }

    #[test]
    fn degraded_paths_report_fixed_literals_without_response_content() {
        let unreadable_balance = "balance-body-that-must-not-be-quoted";
        let responses = parse_responses(
            &json!({ "account": { "type": "chatgpt", "email": EMAIL_MARKER, "planType": "unknown" }, "requiresOpenaiAuth": true }),
            &json!({ "rateLimitsByLimitId": { "codex": { "credits": { "hasCredits": true, "unlimited": false, "balance": unreadable_balance } } } }),
            &json!({ "summary": {} }),
        );
        let data = build_source_data(&responses, collected_at(), data_as_of());
        let info = &data.structured;

        assert_eq!(info.diagnostics.len(), 3);
        for entry in &info.diagnostics {
            assert!(!entry.contains(unreadable_balance));
            assert!(!entry.contains(EMAIL_MARKER));
        }
        assert!(!info.status.data_available);
        assert!(!info
            .status
            .message
            .as_deref()
            .expect("no-data message")
            .contains(EMAIL_MARKER));
    }
}
