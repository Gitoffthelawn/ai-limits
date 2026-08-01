use chrono::Utc;

use crate::types::{
    LimitInfo, MoneyUsage, SourceData, SourceStatus, StructuredSourceInfo, UsageInfo,
};

use super::parse::parse_cursor_api_fields;

const PROVIDER: &str = "cursor";
const SOURCE: &str = "cursor_api2";
const SOURCE_LINK: &str = "docs/get-limits/providers/cursor.md";

pub fn build_source_data(response: &str) -> SourceData {
    let collected_at = utc_now();
    let parsed = parse_cursor_api_fields(response);

    if parsed.is_empty() {
        return SourceData {
            raw: Some(response.to_string()),
            structured: StructuredSourceInfo {
                provider: PROVIDER.to_string(),
                source: SOURCE.to_string(),
                source_link: SOURCE_LINK.to_string(),
                status: SourceStatus {
                    data_available: false,
                    access_available: true,
                    message: Some(
                        "Cursor api2 usage unavailable: response format is not recognized"
                            .to_string(),
                    ),
                    cli_authorization: None,
                },
                raw_data_available: true,
                collected_at: Some(collected_at),
                data_as_of: None,
                account: Default::default(),
                limits: Vec::new(),
                available_limit_resets: None,
                usage: UsageInfo::default(),
                diagnostics: Vec::new(),
            },
            stderr: String::new(),
        };
    }

    let mut limits = Vec::new();
    let mut diagnostics = Vec::new();

    if parsed.remaining.is_some() || parsed.limit.is_some() || parsed.total_percent_used.is_some() {
        let (used_amount, remaining_amount, total_amount) = fill_amount_triple(
            parsed
                .limit
                .zip(parsed.remaining)
                .map(|(limit, remaining)| (limit - remaining).max(0.0)),
            parsed.remaining,
            parsed.limit,
        );

        let (used_percent, remaining_percent) = match parsed.total_percent_used {
            Some(used) => (Some(used), Some(complementary_percent(used))),
            None => (None, None),
        };

        limits.push(LimitInfo {
            name: "plan_usage".to_string(),
            window_label: billing_cycle_label(parsed.billing_cycle_start, parsed.billing_cycle_end),
            window_minutes: None,
            resets_at: parsed.billing_cycle_end.map(format_unix_ms_timestamp),
            used_percent,
            remaining_percent,
            used_amount: cents_to_usd(used_amount),
            remaining_amount: cents_to_usd(remaining_amount),
            total_amount: cents_to_usd(total_amount),
            amount_unit: Some("usd".to_string()),
        });
    }

    if let Some(used_percent) = parsed.auto_percent_used {
        limits.push(percent_limit("auto", used_percent));
    }

    if let Some(used_percent) = parsed.api_percent_used {
        limits.push(percent_limit("api_models", used_percent));
    }

    if parsed.remaining.is_none() && parsed.limit.is_none() && parsed.total_percent_used.is_some() {
        diagnostics.push(
            "plan usage amounts are unavailable; only totalPercentUsed is present".to_string(),
        );
    }

    let (money_used, money_remaining, money_total) = fill_amount_triple(
        parsed
            .limit
            .zip(parsed.remaining)
            .map(|(limit, remaining)| (limit - remaining).max(0.0)),
        parsed.remaining,
        parsed.limit,
    );

    SourceData {
        raw: Some(response.to_string()),
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: parsed.display_message.clone(),
                cli_authorization: None,
            },
            raw_data_available: true,
            collected_at: Some(collected_at.clone()),
            data_as_of: Some(collected_at),
            account: Default::default(),
            limits,
            available_limit_resets: None,
            usage: UsageInfo {
                money: MoneyUsage {
                    used_amount: cents_to_usd(money_used),
                    remaining_amount: cents_to_usd(money_remaining),
                    total_amount: cents_to_usd(money_total),
                    currency: Some("USD".to_string()),
                },
                ..UsageInfo::default()
            },
            diagnostics,
        },
        stderr: String::new(),
    }
}

pub(super) fn access_denied(message: String, raw: Option<String>) -> SourceData {
    SourceData {
        raw: raw.clone(),
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some(message),
                cli_authorization: None,
            },
            raw_data_available: raw.is_some(),
            collected_at: Some(utc_now()),
            data_as_of: None,
            account: Default::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        },
        stderr: String::new(),
    }
}

fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn percent_limit(name: &str, used_percent: f64) -> LimitInfo {
    LimitInfo {
        name: name.to_string(),
        window_label: None,
        window_minutes: None,
        resets_at: None,
        used_percent: Some(used_percent),
        remaining_percent: Some(complementary_percent(used_percent)),
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    }
}

fn complementary_percent(used_percent: f64) -> f64 {
    (100.0 - used_percent).clamp(0.0, 100.0)
}

fn fill_amount_triple(
    used: Option<f64>,
    remaining: Option<f64>,
    total: Option<f64>,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    match (used, remaining, total) {
        (None, Some(remaining), Some(total)) => (
            Some((total - remaining).max(0.0)),
            Some(remaining),
            Some(total),
        ),
        (Some(used), None, Some(total)) => (Some(used), Some((total - used).max(0.0)), Some(total)),
        (Some(used), Some(remaining), None) => {
            (Some(used), Some(remaining), Some(used + remaining))
        }
        other => other,
    }
}

fn cents_to_usd(value: Option<f64>) -> Option<f64> {
    value.map(|amount| amount / 100.0)
}

fn billing_cycle_label(start: Option<i64>, end: Option<i64>) -> Option<String> {
    match (start, end) {
        (Some(start), Some(end)) => Some(format!(
            "{} -> {}",
            format_unix_ms_date(start),
            format_unix_ms_date(end)
        )),
        _ => None,
    }
}

fn format_unix_ms_timestamp(value: i64) -> String {
    let seconds = value.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    civil_date_from_days(days)
        .map(|(year, month, day)| format!("{year:04}-{month:02}-{day:02}T00:00:00Z"))
        .unwrap_or_else(|| value.to_string())
}

fn format_unix_ms_date(value: i64) -> String {
    let seconds = value.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    civil_date_from_days(days)
        .map(|(year, month, day)| format!("{year:04}-{month:02}-{day:02}"))
        .unwrap_or_else(|| value.to_string())
}

fn civil_date_from_days(days_since_unix_epoch: i64) -> Option<(i32, u32, u32)> {
    let days = days_since_unix_epoch.checked_add(719_468)?;
    let era = if days >= 0 { days } else { days - 146_096 }.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        .div_euclid(365);
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2).div_euclid(153);
    let day = day_of_year - (153 * month_prime + 2).div_euclid(5) + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let adjusted_year = year + if month <= 2 { 1 } else { 0 };

    Some((adjusted_year as i32, month as u32, day as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"{
  "planUsage": {
    "remaining": 2000,
    "limit": 2000,
    "autoPercentUsed": 0,
    "apiPercentUsed": 0,
    "totalPercentUsed": 0
  },
  "displayMessage": "You've used 0% of your included usage",
  "billingCycleStart": "1782614703000",
  "billingCycleEnd": "1785206703000"
}"#;

    #[test]
    fn builds_structured_data_from_representative_response() {
        let result = build_source_data(SAMPLE_RESPONSE);
        let structured = &result.structured;

        assert_eq!(structured.provider, "cursor");
        assert_eq!(structured.source, "cursor_api2");
        assert!(structured.status.access_available);
        assert!(structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("You've used 0% of your included usage")
        );
        assert!(structured.raw_data_available);
        assert_eq!(result.raw.as_deref(), Some(SAMPLE_RESPONSE));
        assert_eq!(
            structured.data_as_of.as_deref(),
            structured.collected_at.as_deref()
        );

        assert_eq!(structured.limits.len(), 3);

        let plan = structured
            .limits
            .iter()
            .find(|limit| limit.name == "plan_usage")
            .expect("plan_usage limit");
        assert_eq!(plan.used_percent, Some(0.0));
        assert_eq!(plan.remaining_percent, Some(100.0));
        assert_eq!(plan.used_amount, Some(0.0));
        assert_eq!(plan.remaining_amount, Some(20.0));
        assert_eq!(plan.total_amount, Some(20.0));
        assert_eq!(plan.amount_unit.as_deref(), Some("usd"));
        assert_eq!(
            plan.window_label.as_deref(),
            Some("2026-06-28 -> 2026-07-28")
        );
        assert_eq!(plan.resets_at.as_deref(), Some("2026-07-28T00:00:00Z"));

        let auto = structured
            .limits
            .iter()
            .find(|limit| limit.name == "auto")
            .expect("auto limit");
        assert_eq!(auto.used_percent, Some(0.0));
        assert_eq!(auto.remaining_percent, Some(100.0));

        let api = structured
            .limits
            .iter()
            .find(|limit| limit.name == "api_models")
            .expect("api_models limit");
        assert_eq!(api.used_percent, Some(0.0));
        assert_eq!(api.remaining_percent, Some(100.0));

        assert_eq!(structured.usage.money.used_amount, Some(0.0));
        assert_eq!(structured.usage.money.remaining_amount, Some(20.0));
        assert_eq!(structured.usage.money.total_amount, Some(20.0));
        assert_eq!(structured.usage.money.currency.as_deref(), Some("USD"));
    }

    #[test]
    fn represents_access_denied_without_raw_data() {
        let result = access_denied(
            "Cursor api2 usage unavailable: token not found; run `cursor agent login`".to_string(),
            None,
        );

        assert!(!result.structured.status.access_available);
        assert!(!result.structured.status.data_available);
        assert!(!result.structured.raw_data_available);
        assert!(result.raw.is_none());
        assert!(result.structured.limits.is_empty());
    }

    #[test]
    fn represents_unauthenticated_response_with_raw_data() {
        let raw = r#"{"code":"unauthenticated","message":"invalid token"}"#;
        let result = access_denied(
            "Cursor api2 usage unavailable: token rejected; run `cursor agent login`".to_string(),
            Some(raw.to_string()),
        );

        assert!(!result.structured.status.access_available);
        assert!(!result.structured.status.data_available);
        assert!(result.structured.raw_data_available);
        assert_eq!(result.raw.as_deref(), Some(raw));
    }

    #[test]
    fn marks_unrecognized_response_as_accessible_without_data() {
        let raw = r#"{"unexpected":"shape"}"#;
        let result = build_source_data(raw);

        assert!(result.structured.status.access_available);
        assert!(!result.structured.status.data_available);
        assert!(result.structured.raw_data_available);
        assert_eq!(
            result.structured.status.message.as_deref(),
            Some("Cursor api2 usage unavailable: response format is not recognized")
        );
    }

    #[test]
    fn calculates_remaining_percent_from_used_percent() {
        let raw =
            r#"{"planUsage":{"totalPercentUsed":37.5,"autoPercentUsed":10,"apiPercentUsed":5}}"#;
        let result = build_source_data(raw);
        let plan = result
            .structured
            .limits
            .iter()
            .find(|limit| limit.name == "plan_usage")
            .expect("plan_usage limit");

        assert_eq!(plan.used_percent, Some(37.5));
        assert_eq!(plan.remaining_percent, Some(62.5));
        assert!(result
            .structured
            .diagnostics
            .iter()
            .any(|entry| { entry.contains("plan usage amounts are unavailable") }));
    }

    #[test]
    fn fills_missing_plan_amount_from_limit_and_remaining() {
        let raw = r#"{"planUsage":{"remaining":1500,"limit":2000,"totalPercentUsed":25}}"#;
        let result = build_source_data(raw);
        let plan = result
            .structured
            .limits
            .iter()
            .find(|limit| limit.name == "plan_usage")
            .expect("plan_usage limit");

        assert_eq!(plan.used_amount, Some(5.0));
        assert_eq!(plan.remaining_amount, Some(15.0));
        assert_eq!(plan.total_amount, Some(20.0));
        assert_eq!(result.structured.usage.money.used_amount, Some(5.0));
    }
}
