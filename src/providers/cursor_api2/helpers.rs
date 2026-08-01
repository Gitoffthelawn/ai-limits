use chrono::Utc;

use crate::types::LimitInfo;

pub(super) fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub(super) fn percent_limit(name: &str, used_percent: f64) -> LimitInfo {
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

pub(super) fn complementary_percent(used_percent: f64) -> f64 {
    (100.0 - used_percent).clamp(0.0, 100.0)
}

pub(super) fn fill_amount_triple(
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

pub(super) fn cents_to_usd(value: Option<f64>) -> Option<f64> {
    value.map(|amount| amount / 100.0)
}

pub(super) fn billing_cycle_label(start: Option<i64>, end: Option<i64>) -> Option<String> {
    match (start, end) {
        (Some(start), Some(end)) => Some(format!(
            "{} -> {}",
            format_unix_ms_date(start),
            format_unix_ms_date(end)
        )),
        _ => None,
    }
}

pub(super) fn format_unix_ms_timestamp(value: i64) -> String {
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
