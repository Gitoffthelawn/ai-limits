use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// The two windows this source projects. Their length is fixed by their own
/// names, so no window duration has to be inferred.
pub(super) const FIVE_HOUR_WINDOW: &str = "five_hour";
pub(super) const SEVEN_DAY_WINDOW: &str = "seven_day";
pub(super) const FIVE_HOUR_WINDOW_MINUTES: u64 = 300;
pub(super) const SEVEN_DAY_WINDOW_MINUTES: u64 = 10080;

/// `rate_limits.limits[].kind` values that describe the same windows.
pub(super) const FIVE_HOUR_LIMIT_KIND: &str = "session";
pub(super) const SEVEN_DAY_LIMIT_KIND: &str = "weekly_all";

pub(super) const CRITICAL_SEVERITY: &str = "critical";
pub(super) const NORMAL_SEVERITY: &str = "normal";
pub(super) const USD_CURRENCY: &str = "USD";

/// The declared `subscription_type` enum. Anything else is not a plan name.
const SUBSCRIPTION_TIERS: [&str; 4] = ["pro", "max", "team", "enterprise"];

/// The largest minor-unit scale that is treated as a currency scale rather
/// than as a changed contract.
const MAX_DECIMAL_PLACES: u32 = 6;

/// The `get_usage` payload, reduced to the keys documented in
/// [claude-rpc-usage.md](../../../docs/get-limits/providers/claude-rpc-usage.md).
///
/// Every key outside these DTOs is dropped at parse time, so raw data is a
/// re-serialization of what was read and never a copy of the wire response.
/// The method is officially experimental and its schema is compiled into the
/// CLI binary, so an unknown shape degrades the affected field group to `None`
/// instead of being read positionally or by a closest-matching key.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct ClaudeUsageResponse {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) session: Option<SessionUsage>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) subscription_type: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) rate_limits_available: Option<bool>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) rate_limits: Option<RateLimits>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) behaviors: Option<Behaviors>,
}

/// The usage of the collector's own CLI process, not of the account.
///
/// It is never projected into `usage.*`. `model_usage` is deliberately absent
/// from this DTO: it is keyed by model with an unobserved value shape, it
/// describes this one-shot process and is therefore empty by construction, and
/// nothing in the structured schema is fed from it.
#[derive(Debug, Deserialize, Serialize)]
pub(super) struct SessionUsage {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) total_cost_usd: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) total_api_duration_ms: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) total_duration_ms: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) total_lines_added: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) total_lines_removed: Option<i64>,
}

/// The server payload passed through by the CLI.
///
/// Only the named windows below are read. The code-named entries — `tangelo`,
/// `iguana_necktie`, `omelette_promotional`, `nimbus_quill`, `cinder_cove`,
/// `amber_ladder`, `seven_day_omelette`, and any future sibling — have an
/// open-ended key set and unknown semantics, so they are not read at all.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct RateLimits {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) five_hour: Option<RateLimitWindow>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) seven_day: Option<RateLimitWindow>,
    /// Model- and surface-scoped windows. Readable and kept for raw data, but
    /// not projected: they are not the account's headline quota.
    #[serde(default, deserialize_with = "lenient")]
    pub(super) seven_day_opus: Option<RateLimitWindow>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) seven_day_sonnet: Option<RateLimitWindow>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) seven_day_oauth_apps: Option<RateLimitWindow>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) seven_day_cowork: Option<RateLimitWindow>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) extra_usage: Option<ExtraUsage>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) limits: Option<Vec<LimitRecord>>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) spend: Option<Spend>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) member_dashboard_available: Option<bool>,
    /// Declared by the schema, absent from the observed response. Kept for raw
    /// data only.
    #[serde(default, deserialize_with = "lenient")]
    pub(super) model_scoped: Option<Vec<ModelScopedWindow>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct RateLimitWindow {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) utilization: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) resets_at: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) limit_dollars: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) used_dollars: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) remaining_dollars: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ModelScopedWindow {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) display_name: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) utilization: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) resets_at: Option<String>,
}

/// The paid overflow allowance. Amounts are integers in minor units and
/// `decimal_places` gives the scale.
///
/// `disabled_reason` is deliberately absent: it is backend copy that may name
/// the account's organization, and the disabled state alone is what the
/// projection reports.  `daily` and `weekly` are absent because their value
/// shape was never observed and guessing keys is not allowed.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct ExtraUsage {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) is_enabled: Option<bool>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) monthly_limit: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) used_credits: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) utilization: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) currency: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) decimal_places: Option<u32>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) user_disabled: Option<bool>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) spend_limit_reached: Option<bool>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) credits_ever_enabled: Option<bool>,
}

/// A flat active-limit state. It carries no absolute amounts; it supplies
/// `severity` and `is_active` for the matching named window and nothing else.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct LimitRecord {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) kind: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) group: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) percent: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) severity: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) resets_at: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) scope: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) is_active: Option<bool>,
}

/// The account's own spend state.
///
/// `disclaimer` is deliberately absent: it is backend copy carrying a support
/// article link, which is a help page and must never be used as a management
/// link for the authenticated account. `disabled_reason` is absent for the
/// same reason as in `ExtraUsage`, and `auto_reload` because its value shape
/// was never observed.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct Spend {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) used: Option<SpendAmount>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) limit: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) percent: Option<f64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) severity: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) enabled: Option<bool>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) cap: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) balance: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) can_purchase_credits: Option<bool>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) can_toggle: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct SpendAmount {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) amount_minor: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) currency: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) exponent: Option<u32>,
}

/// A local, windowed, approximate scan of the transcripts on this machine.
///
/// Only the two counts are read. `behaviors[]`, `agents[]`, `skills[]`,
/// `plugins[]`, and `mcp_servers[]` name the user's own local agents, skills,
/// plugins, MCP servers, and working patterns, so they are not fields of this
/// DTO at all and cannot reach structured data, raw data, or diagnostics.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct Behaviors {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) day: Option<BehaviorWindow>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) week: Option<BehaviorWindow>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct BehaviorWindow {
    #[serde(default, deserialize_with = "lenient")]
    pub(super) request_count: Option<i64>,
    #[serde(default, deserialize_with = "lenient")]
    pub(super) session_count: Option<i64>,
}

/// Reads one member without letting its shape decide the fate of its
/// siblings. A member that no longer matches the documented shape becomes
/// `None` on its own, at every nesting level, instead of failing the whole
/// structure or being read by a closest-matching key.
fn lenient<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).ok())
}

/// A payload whose top level is not an object degrades the whole response to
/// `None`; a changed member degrades only its own field group.
pub(super) fn parse_usage_response(payload: &Value) -> Option<ClaudeUsageResponse> {
    payload.is_object().then(|| {
        serde_json::from_value(payload.clone()).unwrap_or_else(|_| ClaudeUsageResponse::default())
    })
}

impl ClaudeUsageResponse {
    pub(super) fn limit_record(&self, kind: &str) -> Option<&LimitRecord> {
        self.rate_limits
            .as_ref()?
            .limits
            .as_ref()?
            .iter()
            .find(|record| record.kind.as_deref() == Some(kind))
    }
}

/// The plan tier as reported, restricted to the declared enum. An unrecognized
/// value is not a plan name and is never passed through.
pub(super) fn plan_name(subscription_type: &str) -> Option<&str> {
    SUBSCRIPTION_TIERS
        .contains(&subscription_type)
        .then_some(subscription_type)
}

/// Server-reported ISO 8601 normalized to ISO 8601 UTC without sub-second
/// noise. No date reconstruction and no timezone guessing are involved.
pub(super) fn format_iso_utc(value: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| format_utc(timestamp.with_timezone(&Utc)))
}

pub(super) fn format_utc(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// A minor-unit integer scaled by the reported number of decimal places. An
/// implausible scale is a changed contract, not a very large amount.
pub(super) fn scale_minor_units(amount: i64, decimal_places: u32) -> Option<f64> {
    (decimal_places <= MAX_DECIMAL_PLACES)
        .then(|| amount as f64 / 10_f64.powi(decimal_places as i32))
}

pub(super) fn remaining_percent(used_percent: f64) -> f64 {
    let remaining = (100.0 - used_percent).max(0.0);
    (remaining * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn verified_payload() -> Value {
        json!({
            "session": { "total_cost_usd": 0, "total_duration_ms": 1623, "model_usage": {} },
            "subscription_type": "pro",
            "rate_limits_available": true,
            "rate_limits": {
                "five_hour": {
                    "utilization": 100,
                    "resets_at": "2026-08-03T22:29:59.402550+00:00",
                    "limit_dollars": null,
                    "used_dollars": null,
                    "remaining_dollars": null
                },
                "seven_day": { "utilization": 31, "resets_at": "2026-08-04T09:59:59.402584+00:00" },
                "tangelo": null,
                "limits": [
                    { "kind": "session", "percent": 100, "severity": "critical", "is_active": true },
                    { "kind": "weekly_all", "percent": 31, "severity": "normal", "is_active": false }
                ]
            }
        })
    }

    #[test]
    fn parses_the_verified_payload() {
        let response = parse_usage_response(&verified_payload()).expect("payload parses");
        let rate_limits = response.rate_limits.as_ref().expect("rate limits");

        assert_eq!(response.subscription_type.as_deref(), Some("pro"));
        assert_eq!(response.rate_limits_available, Some(true));
        assert_eq!(
            rate_limits
                .five_hour
                .as_ref()
                .and_then(|window| window.utilization),
            Some(100.0)
        );
        assert_eq!(
            response
                .limit_record(SEVEN_DAY_LIMIT_KIND)
                .map(|record| record.percent),
            Some(Some(31.0))
        );
    }

    #[test]
    fn code_named_windows_are_not_read_at_any_nesting_level() {
        let response = parse_usage_response(&json!({
            "rate_limits": {
                "tangelo": { "utilization": 5 },
                "iguana_necktie": { "utilization": 6 },
                "seven_day_omelette": { "utilization": 7 }
            }
        }))
        .expect("payload parses");

        let raw = serde_json::to_string(&response).expect("serializes");
        for key in ["tangelo", "iguana_necktie", "omelette"] {
            assert!(!raw.contains(key));
        }
    }

    #[test]
    fn a_non_object_payload_degrades_the_whole_response() {
        assert!(parse_usage_response(&json!("changed")).is_none());
        assert!(parse_usage_response(&json!([1, 2, 3])).is_none());
    }

    #[test]
    fn a_changed_member_degrades_only_its_own_group() {
        let response = parse_usage_response(&json!({
            "subscription_type": "max",
            "rate_limits": "changed"
        }))
        .expect("payload parses");

        assert_eq!(response.subscription_type.as_deref(), Some("max"));
        assert!(response.rate_limits.is_none());
    }

    #[test]
    fn only_declared_subscription_tiers_are_plan_names() {
        for tier in ["pro", "max", "team", "enterprise"] {
            assert_eq!(plan_name(tier), Some(tier));
        }
        assert_eq!(plan_name("free"), None);
        assert_eq!(plan_name(""), None);
    }

    #[test]
    fn reset_times_are_normalized_to_iso_utc() {
        assert_eq!(
            format_iso_utc("2026-08-03T22:29:59.402550+00:00").as_deref(),
            Some("2026-08-03T22:29:59Z")
        );
        assert_eq!(
            format_iso_utc("2026-08-03T20:00:00+02:00").as_deref(),
            Some("2026-08-03T18:00:00Z")
        );
        assert_eq!(format_iso_utc("tomorrow"), None);
    }

    #[test]
    fn minor_units_are_scaled_by_the_reported_decimal_places() {
        assert_eq!(scale_minor_units(4314, 2), Some(43.14));
        assert_eq!(scale_minor_units(4314, 0), Some(4314.0));
        assert_eq!(scale_minor_units(4314, 9), None);
    }

    #[test]
    fn remaining_percent_is_derived_from_used_percent() {
        assert_eq!(remaining_percent(100.0), 0.0);
        assert_eq!(remaining_percent(31.0), 69.0);
        assert_eq!(remaining_percent(140.0), 0.0);
    }
}
