use crate::types::LimitInfo;

pub fn remaining_percent_for_display(limit: &LimitInfo) -> Option<f64> {
    limit.remaining_percent.or_else(|| {
        limit
            .used_percent
            .map(|used| (100.0 - used).clamp(0.0, 100.0))
    })
}

pub fn format_decimal(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

pub fn normalize_percent(value: f64) -> f64 {
    let clamped = value.clamp(0.0, 100.0);
    (clamped * 10.0).round() / 10.0
}

pub fn format_percent(value: f64) -> String {
    format!("{:.1}", normalize_percent(value))
}

pub fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::new();

    for (index, character) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(character);
    }

    formatted.chars().rev().collect()
}
