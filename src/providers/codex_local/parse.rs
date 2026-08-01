use serde_json::Value;

use super::raw::{
    CodexLocalRateLimitWindow, CodexLocalRateLimits, CodexLocalTokenTotals, TokenEvent,
};

pub(super) fn parse_token_event(line: &str) -> Option<TokenEvent> {
    let record = serde_json::from_str::<Value>(line).ok()?;
    if !is_token_count_event(&record) {
        return None;
    }

    let usage = parse_token_usage(&record);
    let rate_limits = parse_rate_limits(&record);

    if usage.is_none() && rate_limits.is_none() {
        return None;
    }

    Some(TokenEvent {
        timestamp: record
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        usage,
        rate_limits,
    })
}

fn parse_token_usage(record: &Value) -> Option<CodexLocalTokenTotals> {
    let usage_value = record
        .get("last_token_usage")
        .or_else(|| record.pointer("/payload/info/last_token_usage"))?;

    Some(CodexLocalTokenTotals {
        input_tokens: number_u64(usage_value, "input_tokens")?,
        cached_input_tokens: number_u64(usage_value, "cached_input_tokens").unwrap_or(0),
        output_tokens: number_u64(usage_value, "output_tokens")?,
        reasoning_output_tokens: number_u64(usage_value, "reasoning_output_tokens").unwrap_or(0),
        total_tokens: number_u64(usage_value, "total_tokens")?,
    })
}

fn is_token_count_event(record: &Value) -> bool {
    record.get("type").and_then(Value::as_str) == Some("token_count")
        || (record.get("type").and_then(Value::as_str) == Some("event_msg")
            && record.pointer("/payload/type").and_then(Value::as_str) == Some("token_count"))
}

fn parse_rate_limits(record: &Value) -> Option<CodexLocalRateLimits> {
    let value = record
        .get("rate_limits")
        .or_else(|| record.pointer("/payload/rate_limits"))?;

    let (credits, credits_unlimited) = parse_credits(value.get("credits"));

    Some(CodexLocalRateLimits {
        primary: parse_rate_limit_window(value.get("primary")),
        secondary: parse_rate_limit_window(value.get("secondary")),
        credits,
        credits_unlimited,
        plan_type: value
            .get("plan_type")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

pub(super) fn parse_credits(value: Option<&Value>) -> (Option<f64>, bool) {
    let Some(value) = value else {
        return (None, false);
    };

    if let Some(number) = number_f64_any(value) {
        return (Some(number), false);
    }

    let Some(object) = value.as_object() else {
        return (None, false);
    };

    if object.get("has_credits").and_then(Value::as_bool) == Some(false) {
        return (None, false);
    }

    if object.get("unlimited").and_then(Value::as_bool) == Some(true) {
        return (None, true);
    }

    (object.get("balance").and_then(number_f64_any), false)
}

fn parse_rate_limit_window(value: Option<&Value>) -> Option<CodexLocalRateLimitWindow> {
    let value = value?;
    let used_percent = value.get("used_percent").and_then(number_f64_any);
    let window_minutes = value.get("window_minutes").and_then(number_u64_any);
    let resets_at = value.get("resets_at").and_then(number_u64_any);

    if used_percent.is_none() && window_minutes.is_none() && resets_at.is_none() {
        return None;
    }

    Some(CodexLocalRateLimitWindow {
        used_percent,
        window_minutes,
        resets_at,
    })
}

fn number_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(number_u64_any)
}

fn number_u64_any(value: &Value) -> Option<u64> {
    if let Some(number) = value.as_u64() {
        return Some(number);
    }
    value.as_str().and_then(|raw| raw.parse::<u64>().ok())
}

fn number_f64_any(value: &Value) -> Option<f64> {
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    value.as_str().and_then(|raw| raw.parse::<f64>().ok())
}
