use chrono::{DateTime, Utc};
use serde_json::Value;

use super::model::{ServerResetAnchor, TurnUsage};

pub(super) fn extract_turn_usage(record: &Value) -> Option<TurnUsage> {
    if record.get("type")?.as_str()? != "assistant" {
        return None;
    }

    let session_id = record.get("sessionId")?.as_str()?.to_string();
    let message = record.get("message")?;
    let usage = message.get("usage")?;
    let input_tokens = number_field(usage, "input_tokens");
    let output_tokens = number_field(usage, "output_tokens");
    let cache_read_tokens = number_field(usage, "cache_read_input_tokens");
    let cache_creation_tokens = number_field(usage, "cache_creation_input_tokens");

    if input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens == 0 {
        return None;
    }

    Some(TurnUsage {
        session_id,
        timestamp: record
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        model: message
            .get("model")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        message_id: message
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
    })
}

fn number_field(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

pub(super) fn extract_server_reset_anchor(record: &Value) -> Option<ServerResetAnchor> {
    let mut candidates = Vec::new();
    collect_server_reset_anchor_candidates(record, "", false, &mut candidates);
    candidates.into_iter().max()
}

fn collect_server_reset_anchor_candidates(
    value: &Value,
    path: &str,
    in_reset_context: bool,
    candidates: &mut Vec<ServerResetAnchor>,
) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = join_json_path(path, key);
                let key_is_reset_context = in_reset_context || is_server_reset_context_key(key);

                if is_reset_timestamp(key) && key_is_reset_context {
                    if let Some(resets_at) = parse_reset_timestamp_value(child) {
                        candidates.push(ServerResetAnchor {
                            resets_at,
                            source_path: child_path.clone(),
                        });
                    }
                }

                collect_server_reset_anchor_candidates(
                    child,
                    &child_path,
                    key_is_reset_context,
                    candidates,
                );
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                collect_server_reset_anchor_candidates(
                    child,
                    &child_path,
                    in_reset_context,
                    candidates,
                );
            }
        }
        _ => {}
    }
}

fn join_json_path(path: &str, key: &str) -> String {
    let escaped = key.replace('~', "~0").replace('/', "~1");
    if path.is_empty() {
        format!("/{escaped}")
    } else {
        format!("{path}/{escaped}")
    }
}

fn is_server_reset_context_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    normalized.contains("ratelimit")
        || normalized.contains("usagelimit")
        || normalized.contains("usage")
        || normalized.contains("quota")
        || normalized.contains("429")
}

fn is_reset_timestamp(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    matches!(
        normalized.as_str(),
        "resetsat" | "resetat" | "resettime" | "resettimestamp" | "limitresetat"
    )
}

fn parse_reset_timestamp_value(value: &Value) -> Option<DateTime<Utc>> {
    if let Some(timestamp) = value.as_u64() {
        return DateTime::from_timestamp(timestamp as i64, 0);
    }

    if let Some(timestamp) = value.as_i64() {
        return DateTime::from_timestamp(timestamp, 0);
    }

    let text = value.as_str()?.trim();
    if text.is_empty() {
        return None;
    }

    if let Ok(timestamp) = text.parse::<i64>() {
        return DateTime::from_timestamp(timestamp, 0);
    }

    parse_timestamp(text)
}

pub(super) fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn extracts_server_reset_anchor_from_rate_limits_payload() {
        let record: Value = serde_json::from_str(
            r#"{"type":"assistant","payload":{"rate_limits":{"five_hour":{"resets_at":"1782721800"}}}}"#,
        )
        .expect("parse record");

        let anchor = extract_server_reset_anchor(&record).expect("server reset anchor");

        assert_eq!(
            anchor
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-29T08:30:00Z"
        );
        assert_eq!(
            anchor.source_path,
            "/payload/rate_limits/five_hour/resets_at"
        );
    }

    #[test]
    fn extracts_server_reset_anchor_from_nested_429_usage_limit_payload() {
        let record: Value = serde_json::from_str(
            r#"{"type":"error","payload":{"status":429,"error":{"usage_limit":{"reset_time":"2026-06-29T08:30:00Z"}}}}"#,
        )
        .expect("parse record");

        let anchor = extract_server_reset_anchor(&record).expect("server reset anchor");

        assert_eq!(
            anchor
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-29T08:30:00Z"
        );
        assert_eq!(anchor.source_path, "/payload/error/usage_limit/reset_time");
    }
}
