use serde_json::Value;

use super::raw::{
    CodexLocalRateLimitWindow, CodexLocalRateLimits, CodexLocalTokenTotals, TokenEvent,
};

const TOKEN_COUNT: &str = "token_count";
const SESSION_META: &str = "session_meta";
const TASK_STARTED: &str = "task_started";
const TASK_COMPLETE: &str = "task_complete";
const PATCH_APPLY_END: &str = "patch_apply_end";

/// One JSONL record that carries a business fact for this source.
///
/// `Session` and `Turn` hold identifiers and `Changes` holds absolute paths
/// from the user's file system: the scan reduces all three to counts of
/// distinct values and never keeps the values themselves.
pub(super) enum CodexLocalRecord {
    TokenCount(TokenEvent),
    Session(String),
    Turn(String),
    Changes(Vec<String>),
}

pub(super) fn parse_record(line: &str) -> Option<CodexLocalRecord> {
    let record = serde_json::from_str::<Value>(line).ok()?;

    match event_type(&record)? {
        TOKEN_COUNT => parse_token_event(&record).map(CodexLocalRecord::TokenCount),
        SESSION_META => parse_session_id(&record).map(CodexLocalRecord::Session),
        TASK_STARTED | TASK_COMPLETE => parse_turn_id(&record).map(CodexLocalRecord::Turn),
        PATCH_APPLY_END => parse_changed_paths(&record).map(CodexLocalRecord::Changes),
        _ => None,
    }
}

/// Records arrive either wrapped in an `event_msg` envelope or flat, so the
/// payload type wins and the record type is the fallback.
fn event_type(record: &Value) -> Option<&str> {
    record
        .pointer("/payload/type")
        .and_then(Value::as_str)
        .or_else(|| record.get("type").and_then(Value::as_str))
}

fn parse_token_event(record: &Value) -> Option<TokenEvent> {
    let usage = parse_token_usage(record);
    let rate_limits = parse_rate_limits(record);

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

fn parse_session_id(record: &Value) -> Option<String> {
    string_field(record, "session_id")
}

fn parse_turn_id(record: &Value) -> Option<String> {
    string_field(record, "turn_id")
}

fn parse_changed_paths(record: &Value) -> Option<Vec<String>> {
    let changes = record
        .pointer("/payload/changes")
        .or_else(|| record.get("changes"))?
        .as_object()?;

    if changes.is_empty() {
        return None;
    }

    Some(changes.keys().cloned().collect())
}

fn string_field(record: &Value, key: &str) -> Option<String> {
    record
        .pointer(&format!("/payload/{key}"))
        .or_else(|| record.get(key))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn parse_token_usage(record: &Value) -> Option<CodexLocalTokenTotals> {
    let usage_value = record
        .get("last_token_usage")
        .or_else(|| record.pointer("/payload/info/last_token_usage"))?;

    Some(CodexLocalTokenTotals {
        input_tokens: number_u64(usage_value, "input_tokens")?,
        cached_input_tokens: number_u64(usage_value, "cached_input_tokens").unwrap_or(0),
        cache_write_input_tokens: number_u64(usage_value, "cache_write_input_tokens"),
        output_tokens: number_u64(usage_value, "output_tokens")?,
        reasoning_output_tokens: number_u64(usage_value, "reasoning_output_tokens").unwrap_or(0),
        total_tokens: number_u64(usage_value, "total_tokens")?,
    })
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
