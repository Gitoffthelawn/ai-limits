mod parse;
mod process;
mod project;

use std::io;

use chrono::Utc;

use crate::infra::os_access::{allowed_cli_command_is_available, CLAUDE_CLI_COMMAND};
use crate::types::SourceData;

use parse::parse_usage_response;
use process::read_usage;
use project::{build_source_data, unavailable_source_data, CLI_MISSING_MESSAGE};

pub fn collect_usage() -> io::Result<SourceData> {
    if !allowed_cli_command_is_available(CLAUDE_CLI_COMMAND) {
        return Ok(unavailable_source_data(CLI_MISSING_MESSAGE));
    }

    let collected_at = Utc::now();
    let payload = match read_usage() {
        Ok(payload) => payload,
        Err(failure) => return Ok(unavailable_source_data(failure.message())),
    };
    let data_as_of = Utc::now();

    let response = parse_usage_response(&payload);
    Ok(build_source_data(
        response.as_ref(),
        collected_at,
        data_as_of,
    ))
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use serde_json::{json, Value};

    use super::parse::parse_usage_response;
    use super::project::{build_source_data, unavailable_source_data, CLI_MISSING_MESSAGE};
    use crate::types::SourceData;

    /// Recognizable stand-ins for every kind of identifier the verified payload
    /// carries alongside the usage data.
    const EMAIL_MARKER: &str = "account-marker@example-secret.test";
    const ACCOUNT_ID_MARKER: &str = "acct_0123456789abcdef";
    const ORGANIZATION_MARKER: &str = "Marker Organization GmbH";
    const MCP_SERVER_MARKER: &str = "marker-mcp-server";
    const AGENT_MARKER: &str = "marker-agent";
    const SKILL_MARKER: &str = "marker-skill";
    const PLUGIN_MARKER: &str = "marker-plugin";
    const PATH_MARKER: &str = "/Users/marker/Projects/secret-repo";

    const MARKERS: [&str; 8] = [
        EMAIL_MARKER,
        ACCOUNT_ID_MARKER,
        ORGANIZATION_MARKER,
        MCP_SERVER_MARKER,
        AGENT_MARKER,
        SKILL_MARKER,
        PLUGIN_MARKER,
        PATH_MARKER,
    ];

    fn timestamp(value: &str) -> DateTime<chrono::Utc> {
        value.parse().expect("valid timestamp")
    }

    fn collected_at() -> DateTime<chrono::Utc> {
        timestamp("2026-08-03T12:00:00Z")
    }

    fn data_as_of() -> DateTime<chrono::Utc> {
        timestamp("2026-08-03T12:00:02Z")
    }

    /// The shape observed on claude 2.1.220, with every identifying member
    /// replaced by a marker.
    fn verified_payload() -> Value {
        json!({
            "session": {
                "total_cost_usd": 0,
                "total_api_duration_ms": 0,
                "total_duration_ms": 1623,
                "total_lines_added": 0,
                "total_lines_removed": 0,
                "model_usage": { "claude-opus-4": { "costUSD": 0 } },
                "cwd": PATH_MARKER
            },
            "subscription_type": "pro",
            "rate_limits_available": true,
            "rate_limits": {
                "five_hour": {
                    "utilization": 100,
                    "resets_at": "2026-08-03T22:29:59.402550+00:00",
                    "limit_dollars": 40.0,
                    "used_dollars": 40.0,
                    "remaining_dollars": 0.0
                },
                "seven_day": {
                    "utilization": 31,
                    "resets_at": "2026-08-04T09:59:59.402584+00:00",
                    "limit_dollars": null,
                    "used_dollars": null,
                    "remaining_dollars": null
                },
                "seven_day_opus": null,
                "seven_day_sonnet": null,
                "seven_day_oauth_apps": null,
                "seven_day_cowork": null,
                "tangelo": { "utilization": 12, "resets_at": "2026-08-05T00:00:00+00:00" },
                "iguana_necktie": { "utilization": 3 },
                "omelette_promotional": null,
                "nimbus_quill": null,
                "cinder_cove": null,
                "amber_ladder": null,
                "extra_usage": {
                    "is_enabled": true,
                    "monthly_limit": 10000,
                    "used_credits": 4314,
                    "utilization": null,
                    "currency": "EUR",
                    "decimal_places": 2,
                    "disabled_reason": ORGANIZATION_MARKER,
                    "user_disabled": false,
                    "spend_limit_reached": false,
                    "credits_ever_enabled": true,
                    "daily": null,
                    "weekly": null
                },
                "limits": [
                    {
                        "kind": "session",
                        "group": "session",
                        "percent": 100,
                        "severity": "critical",
                        "resets_at": "2026-08-03T22:29:59.402550+00:00",
                        "scope": null,
                        "is_active": true
                    },
                    {
                        "kind": "weekly_all",
                        "group": "weekly",
                        "percent": 31,
                        "severity": "normal",
                        "resets_at": "2026-08-04T09:59:59.402584+00:00",
                        "scope": null,
                        "is_active": false
                    },
                    {
                        "kind": "unrecognized_future_kind",
                        "group": "future",
                        "percent": 7,
                        "severity": "normal",
                        "resets_at": "2026-08-09T00:00:00+00:00",
                        "scope": null,
                        "is_active": true
                    }
                ],
                "spend": {
                    "used": { "amount_minor": 4314, "currency": "EUR", "exponent": 2 },
                    "limit": 10000,
                    "percent": 43,
                    "severity": "normal",
                    "enabled": true,
                    "disabled_reason": null,
                    "cap": null,
                    "balance": null,
                    "auto_reload": null,
                    "disclaimer": "Usage credits cover you when you hit your plan limits.",
                    "can_purchase_credits": false,
                    "can_toggle": false
                },
                "member_dashboard_available": false
            },
            "behaviors": {
                "day": {
                    "request_count": 1719,
                    "session_count": 16,
                    "behaviors": [{ "key": "subagent_heavy", "pct": 66, "count": 2 }],
                    "agents": [{ "name": AGENT_MARKER, "pct": 52 }],
                    "skills": [{ "name": SKILL_MARKER, "pct": 4 }],
                    "plugins": [{ "name": PLUGIN_MARKER, "pct": 2 }],
                    "mcp_servers": [{ "name": MCP_SERVER_MARKER, "pct": 11 }]
                },
                "week": {
                    "request_count": 1939,
                    "session_count": 22,
                    "behaviors": [],
                    "agents": [],
                    "skills": [],
                    "plugins": [],
                    "mcp_servers": []
                }
            },
            "account": {
                "email_address": EMAIL_MARKER,
                "account_uuid": ACCOUNT_ID_MARKER,
                "organization_name": ORGANIZATION_MARKER
            }
        })
    }

    fn source_data_for(payload: &Value) -> SourceData {
        let response = parse_usage_response(payload);
        build_source_data(response.as_ref(), collected_at(), data_as_of())
    }

    fn verified_source_data() -> SourceData {
        source_data_for(&verified_payload())
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
    fn projects_the_verified_response_payload() {
        let data = verified_source_data();
        let info = &data.structured;

        assert_eq!(info.provider, "claude");
        assert_eq!(info.source, "claude_rpc");
        assert!(info.status.access_available);
        assert!(info.status.data_available);
        assert!(info.raw_data_available);
        assert_eq!(info.collected_at.as_deref(), Some("2026-08-03T12:00:00Z"));
        assert_eq!(info.data_as_of.as_deref(), Some("2026-08-03T12:00:02Z"));
        assert_eq!(info.account.plan.as_deref(), Some("pro"));
        assert_eq!(info.account.credits_total, Some(100.0));
        assert_eq!(info.account.credits_used, Some(43.14));
        assert_eq!(info.account.credits_remaining, Some(56.86));
        assert_eq!(info.usage.money.used_amount, Some(43.14));
        assert_eq!(info.usage.money.total_amount, Some(100.0));
        assert_eq!(info.usage.money.remaining_amount, Some(56.86));
        assert_eq!(info.usage.money.currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn the_two_named_windows_become_two_merged_limit_records() {
        let info = verified_source_data().structured;

        assert_eq!(info.limits.len(), 2);
        assert_eq!(info.limits[0].name, "five_hour");
        assert_eq!(info.limits[0].window_minutes, Some(300));
        assert_eq!(info.limits[0].used_percent, Some(100.0));
        assert_eq!(info.limits[0].remaining_percent, Some(0.0));
        assert_eq!(
            info.limits[0].resets_at.as_deref(),
            Some("2026-08-03T22:29:59Z")
        );
        assert_eq!(info.limits[0].used_amount, Some(40.0));
        assert_eq!(info.limits[0].total_amount, Some(40.0));
        assert_eq!(info.limits[0].remaining_amount, Some(0.0));
        assert_eq!(info.limits[0].amount_unit.as_deref(), Some("usd"));

        assert_eq!(info.limits[1].name, "seven_day");
        assert_eq!(info.limits[1].window_minutes, Some(10080));
        assert_eq!(info.limits[1].remaining_percent, Some(69.0));
        assert!(info.limits[1].amount_unit.is_none());
    }

    #[test]
    fn code_named_and_scoped_windows_are_never_projected() {
        let info = verified_source_data().structured;

        for limit in &info.limits {
            assert!(matches!(limit.name.as_str(), "five_hour" | "seven_day"));
        }
        let structured = serde_json::to_string(&info).expect("structured data serializes");
        for key in ["tangelo", "iguana_necktie", "omelette", "opus", "sonnet"] {
            assert!(!structured.contains(key));
        }
    }

    #[test]
    fn an_unrecognized_limit_kind_is_not_projected() {
        let info = verified_source_data().structured;

        assert!(!info
            .limits
            .iter()
            .any(|limit| limit.name.contains("unrecognized")));
        assert_eq!(info.limits.len(), 2);
    }

    #[test]
    fn a_named_window_and_its_limits_entry_never_produce_two_records() {
        let info = source_data_for(&json!({
            "rate_limits_available": true,
            "rate_limits": {
                "five_hour": { "utilization": 80, "resets_at": "2026-08-03T22:00:00+00:00" },
                "limits": [
                    { "kind": "session", "percent": 80, "severity": "normal", "is_active": true }
                ]
            }
        }))
        .structured;

        assert_eq!(info.limits.len(), 1);
        assert_eq!(info.limits[0].used_percent, Some(80.0));
    }

    #[test]
    fn a_missing_window_falls_back_to_the_matching_limits_entry() {
        let info = source_data_for(&json!({
            "rate_limits_available": true,
            "rate_limits": {
                "five_hour": null,
                "limits": [
                    {
                        "kind": "session",
                        "percent": 55,
                        "severity": "normal",
                        "resets_at": "2026-08-03T22:00:00+00:00",
                        "is_active": true
                    }
                ]
            }
        }))
        .structured;

        assert_eq!(info.limits.len(), 1);
        assert_eq!(info.limits[0].used_percent, Some(55.0));
        assert_eq!(
            info.limits[0].resets_at.as_deref(),
            Some("2026-08-03T22:00:00Z")
        );
        assert!(info.limits[0].used_amount.is_none());
    }

    #[test]
    fn the_currency_mismatch_is_recorded_and_never_relabelled_or_converted() {
        let info = verified_source_data().structured;

        assert_eq!(info.usage.money.currency.as_deref(), Some("EUR"));
        assert_eq!(info.limits[0].amount_unit.as_deref(), Some("usd"));
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("currency other than US dollars")));
    }

    #[test]
    fn a_dollar_billed_account_records_no_currency_mismatch() {
        let info = source_data_for(&json!({
            "rate_limits_available": true,
            "rate_limits": {
                "spend": { "used": { "amount_minor": 500, "currency": "USD", "exponent": 2 } }
            }
        }))
        .structured;

        assert_eq!(info.usage.money.used_amount, Some(5.0));
        assert!(!info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("currency other than US dollars")));
    }

    #[test]
    fn a_disabled_extra_usage_allowance_reports_no_credits() {
        let info = source_data_for(&json!({
            "rate_limits_available": true,
            "rate_limits": {
                "extra_usage": {
                    "is_enabled": false,
                    "monthly_limit": 10000,
                    "used_credits": 0,
                    "decimal_places": 2,
                    "disabled_reason": ORGANIZATION_MARKER
                }
            }
        }))
        .structured;

        assert!(info.account.credits_total.is_none());
        assert!(info.account.credits_used.is_none());
        assert!(info.account.credits_remaining.is_none());
        assert_eq!(
            info.diagnostics,
            vec!["credits: the extra usage allowance is disabled".to_string()]
        );
    }

    #[test]
    fn token_activity_and_model_fields_stay_null_with_a_recorded_reason() {
        let info = verified_source_data().structured;

        assert!(info.usage.tokens.total.is_none());
        assert!(info.usage.tokens.input.is_none());
        assert!(info.usage.tokens.output.is_none());
        assert!(info.usage.activity.sessions_count.is_none());
        assert!(info.usage.activity.turns_count.is_none());
        assert!(info.usage.activity.files_count.is_none());
        assert!(info.usage.activity.events_count.is_none());
        assert!(info.usage.activity.latest_activity_at.is_none());
        assert!(info.usage.models.top_model.is_none());
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("windowed local scan of this machine only")));
    }

    #[test]
    fn confirmed_absent_subscription_fields_stay_null() {
        let info = verified_source_data().structured;

        assert!(info.account.subscription_started_at.is_none());
        assert!(info.account.renewal_at.is_none());
        assert!(info.account.price_amount.is_none());
        assert!(info.account.price_currency.is_none());
        assert!(info.account.price_period.is_none());
        assert!(info.account.price_note.is_none());
        assert!(info.account.plan_management_url.is_none());
        assert!(info.account.billing_management_url.is_none());
        assert!(info.available_limit_resets.is_none());
    }

    #[test]
    fn an_unavailable_rate_limit_state_is_not_an_error_and_keeps_the_plan() {
        let data = source_data_for(&json!({
            "subscription_type": "max",
            "rate_limits_available": false,
            "rate_limits": null
        }));
        let info = &data.structured;

        assert!(info.status.access_available);
        assert!(!info.status.data_available);
        assert_eq!(info.account.plan.as_deref(), Some("max"));
        assert!(info.limits.is_empty());
        assert_eq!(
            info.status.message.as_deref(),
            Some("the Claude CLI reports that rate limits are not available for this account")
        );
    }

    #[test]
    fn a_missing_subscription_type_is_a_normal_value_without_a_diagnostic() {
        let info = source_data_for(&json!({
            "subscription_type": null,
            "rate_limits_available": true,
            "rate_limits": { "five_hour": { "utilization": 10 } }
        }))
        .structured;

        assert!(info.account.plan.is_none());
        assert!(!info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("subscription tier")));
    }

    #[test]
    fn an_unrecognized_subscription_tier_degrades_to_null_with_a_diagnostic() {
        let info = source_data_for(&json!({
            "subscription_type": "platinum",
            "rate_limits_available": true
        }))
        .structured;

        assert!(info.account.plan.is_none());
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("subscription tier is not recognized")));
        for entry in &info.diagnostics {
            assert!(!entry.contains("platinum"));
        }
    }

    #[test]
    fn a_changed_response_shape_degrades_to_null_values_plus_a_diagnostic() {
        let data = source_data_for(&json!("changed"));
        let info = &data.structured;

        assert!(info.status.access_available);
        assert!(!info.status.data_available);
        assert!(info.limits.is_empty());
        assert!(info.account.plan.is_none());
        assert!(info.usage.money.used_amount.is_none());
        assert!(info.data_as_of.is_none());
        assert!(data.raw.is_none());
        assert_eq!(
            info.diagnostics,
            vec!["usage: the response could not be read".to_string()]
        );
    }

    #[test]
    fn a_changed_rate_limits_member_degrades_only_the_limits() {
        let info = source_data_for(&json!({
            "subscription_type": "pro",
            "rate_limits_available": true,
            "rate_limits": "changed"
        }))
        .structured;

        assert_eq!(info.account.plan.as_deref(), Some("pro"));
        assert!(info.limits.is_empty());
        assert!(info.usage.money.used_amount.is_none());
        assert!(info.status.data_available);
    }

    #[test]
    fn a_missing_cli_reports_a_fixed_setup_message() {
        let data = unavailable_source_data(CLI_MISSING_MESSAGE);

        assert!(!data.structured.status.access_available);
        assert!(!data.structured.raw_data_available);
        assert!(data.raw.is_none());
        assert_eq!(
            data.structured.status.message.as_deref(),
            Some(CLI_MISSING_MESSAGE)
        );
    }

    #[test]
    fn account_identifiers_and_local_names_never_reach_any_output_artifact() {
        let data = verified_source_data();
        let artifacts = output_artifacts(&data);

        assert!(data.raw.is_some(), "raw data is exposed for this source");
        for artifact in &artifacts {
            for marker in MARKERS {
                assert!(
                    !artifact.contains(marker),
                    "an identifier or local name reached an output artifact"
                );
            }
            for fragment in ["example-secret.test", "marker", "Marker", "/Users/"] {
                assert!(
                    !artifact.contains(fragment),
                    "an identifier fragment reached an output artifact"
                );
            }
        }
    }

    #[test]
    fn degraded_paths_report_fixed_literals_without_response_content() {
        let backend_copy = "disabled-reason-body-that-must-not-be-quoted";
        let data = source_data_for(&json!({
            "subscription_type": "platinum",
            "rate_limits_available": true,
            "rate_limits": {
                "extra_usage": {
                    "is_enabled": true,
                    "monthly_limit": 10000,
                    "decimal_places": 99,
                    "disabled_reason": backend_copy
                },
                "limits": [
                    { "kind": "session", "percent": 5, "severity": "meteor_shower" }
                ],
                "five_hour": { "utilization": 5 },
                "spend": {
                    "used": { "amount_minor": 1, "currency": "EUR", "exponent": 2 },
                    "disclaimer": backend_copy,
                    "disabled_reason": backend_copy
                }
            }
        }));
        let info = &data.structured;

        assert!(info.account.plan.is_none());
        assert!(info.account.credits_total.is_none());
        for entry in &info.diagnostics {
            assert!(!entry.contains(backend_copy));
            assert!(!entry.contains("platinum"));
            assert!(!entry.contains("meteor_shower"));
        }
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("amount scale is missing or unusable")));
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("limit severity is not recognized")));
        for artifact in output_artifacts(&data) {
            assert!(
                !artifact.contains(backend_copy),
                "backend copy reached an output artifact"
            );
        }
    }
}
