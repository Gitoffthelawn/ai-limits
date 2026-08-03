use chrono::{DateTime, Utc};

use crate::types::LimitInfo;

/// Currency symbols the price parser understands. A symbol outside this table
/// makes the whole price string unrecognized.
const CURRENCY_SYMBOLS: &[(&str, &str)] = &[
    ("$", "USD"),
    ("€", "EUR"),
    ("£", "GBP"),
    ("¥", "JPY"),
    ("₹", "INR"),
];

/// Billing period tokens the schema recognizes.
const PERIOD_TOKENS: &[&str] = &["mo", "yr"];

/// A price that parsed as a whole. There is no partial form: the amount, the
/// currency, and the period are always present together.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PlanPrice {
    pub amount: f64,
    pub currency: String,
    pub period: String,
}

/// Parses `planInfo.price`, which packs amount, currency, and period into one
/// string such as `$20/mo`.
///
/// Parsing is all-or-nothing: an amount is never taken without its currency,
/// and a currency is never taken without its amount. Anything the accepted
/// shape does not cover — an unknown symbol, an unknown period, a range, an
/// annotated or localized form — yields `None`, and the caller reports all
/// three fields as unknown.
pub(super) fn parse_price(raw: &str) -> Option<PlanPrice> {
    let (currency, rest) = CURRENCY_SYMBOLS
        .iter()
        .find_map(|(symbol, code)| raw.strip_prefix(symbol).map(|rest| (*code, rest)))?;

    let (amount, period) = rest.split_once('/')?;

    if !PERIOD_TOKENS.contains(&period) {
        return None;
    }

    if amount.is_empty()
        || !amount
            .chars()
            .all(|item| item.is_ascii_digit() || item == '.')
        || amount.matches('.').count() > 1
        || amount.starts_with('.')
        || amount.ends_with('.')
    {
        return None;
    }

    Some(PlanPrice {
        amount: amount.parse::<f64>().ok()?,
        currency: currency.to_string(),
        period: period.to_string(),
    })
}

pub(super) fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub(super) fn percent_limit(name: &str, used_percent: f64, resets_at: Option<String>) -> LimitInfo {
    LimitInfo {
        name: name.to_string(),
        window_label: None,
        window_minutes: None,
        resets_at,
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

/// Percentage of an amount window, used where the source reports money rather
/// than a percentage.
pub(super) fn amount_percents(used: Option<f64>, total: Option<f64>) -> (Option<f64>, Option<f64>) {
    match (used, total) {
        (Some(used), Some(total)) if total > 0.0 => {
            let used_percent = (used / total * 100.0).clamp(0.0, 100.0);
            (
                Some(used_percent),
                Some(complementary_percent(used_percent)),
            )
        }
        _ => (None, None),
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
    DateTime::from_timestamp_millis(value)
        .map(|moment| moment.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| value.to_string())
}

fn format_unix_ms_date(value: i64) -> String {
    DateTime::from_timestamp_millis(value)
        .map(|moment| moment.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_verified_price_string() {
        let price = parse_price("$20/mo").expect("recognized price");

        assert_eq!(price.amount, 20.0);
        assert_eq!(price.currency, "USD");
        assert_eq!(price.period, "mo");
    }

    #[test]
    fn parses_a_decimal_amount_and_a_yearly_period() {
        let price = parse_price("€199.50/yr").expect("recognized price");

        assert_eq!(price.amount, 199.5);
        assert_eq!(price.currency, "EUR");
        assert_eq!(price.period, "yr");
    }

    #[test]
    fn rejects_every_shape_it_does_not_fully_understand() {
        for raw in [
            "",
            "20/mo",
            "$20",
            "$20/month",
            "$20/wk",
            "$20 / mo",
            " $20/mo",
            "$20/mo ",
            "$20-$30/mo",
            "$20.00.00/mo",
            "$./mo",
            "20 USD/mo",
            "₿20/mo",
            "from $20/mo",
            "$20/mo + tax",
            "20,00 €/mo",
        ] {
            assert_eq!(parse_price(raw), None, "{raw} should not parse");
        }
    }

    #[test]
    fn formats_epoch_milliseconds_with_the_time_of_day() {
        assert_eq!(
            format_unix_ms_timestamp(1787878441000),
            "2026-08-28T00:54:01Z"
        );
        assert_eq!(format_unix_ms_date(1787878441000), "2026-08-28");
    }

    #[test]
    fn derives_percentages_from_an_amount_window() {
        assert_eq!(
            amount_percents(Some(5.0), Some(20.0)),
            (Some(25.0), Some(75.0))
        );
        assert_eq!(amount_percents(Some(5.0), None), (None, None));
        assert_eq!(amount_percents(Some(5.0), Some(0.0)), (None, None));
    }
}
