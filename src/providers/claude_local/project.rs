use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use chrono::Utc;
use serde_json::{json, Value};

use crate::types::{
    AccountInfo, ActivityUsage, LimitInfo, ModelUsage, MoneyUsage, SourceStatus,
    StructuredSourceInfo, TokenUsage, UsageInfo,
};

use super::model::{
    active_session_limit, ActiveSessionLimit, ClaudeLocalUsage, ResetSource,
    CLAUDE_LOCAL_SESSION_WINDOW_MINUTES,
};

const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_local";
const SOURCE_LINK: &str = "docs/get-limits";

pub(super) fn encode_raw(
    candidate_roots: &[PathBuf],
    scanned_roots: &[PathBuf],
    usage: Option<&ClaudeLocalUsage>,
) -> io::Result<String> {
    let mut payload = json!({
        "candidate_roots": path_strings(candidate_roots),
        "scanned_roots": path_strings(scanned_roots),
    });

    if let Some(usage) = usage {
        let total_tokens = usage.input_tokens
            + usage.output_tokens
            + usage.cache_read_tokens
            + usage.cache_creation_tokens;
        let mut models = usage
            .models
            .iter()
            .map(|(model, count)| (model.clone(), json!(count)))
            .collect::<Vec<_>>();
        models.sort_by(|(left, _), (right, _)| left.cmp(right));

        payload["usage"] = json!({
            "files": usage.files,
            "sessions": usage.sessions.iter().collect::<Vec<_>>(),
            "turns": usage.turns,
            "input_tokens": usage.input_tokens,
            "output_tokens": usage.output_tokens,
            "cache_read_tokens": usage.cache_read_tokens,
            "cache_creation_tokens": usage.cache_creation_tokens,
            "total_tokens": total_tokens,
            "models": Value::Object(models.into_iter().collect()),
            "latest_timestamp": usage.latest_timestamp,
            "latest_server_reset_anchor": usage.latest_server_reset_anchor.as_ref().map(|anchor| {
                json!({
                    "resets_at": anchor.resets_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    "source_path": anchor.source_path,
                })
            }),
        });
    }

    serde_json::to_string(&payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn path_strings(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

fn structured_base(
    status: SourceStatus,
    raw_data_available: bool,
    data_as_of: Option<String>,
) -> StructuredSourceInfo {
    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status,
        raw_data_available,
        collected_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        data_as_of,
        account: AccountInfo::default(),
        limits: Vec::new(),
        available_limit_resets: None,
        usage: UsageInfo::default(),
        diagnostics: Vec::new(),
    }
}

pub(super) fn structured_no_roots() -> StructuredSourceInfo {
    structured_base(
        SourceStatus {
            data_available: false,
            access_available: true,
            message: Some("local transcript roots were not found".to_string()),
            cli_authorization: None,
        },
        true,
        None,
    )
}

pub(super) fn structured_no_usage(root_count: usize) -> StructuredSourceInfo {
    structured_base(
        SourceStatus {
            data_available: false,
            access_available: true,
            message: Some(format!(
                "no token usage found in {root_count} local transcript root(s)"
            )),
            cli_authorization: None,
        },
        true,
        None,
    )
}

pub(super) fn structured_from_usage(usage: &ClaudeLocalUsage) -> StructuredSourceInfo {
    let total_tokens = usage.input_tokens
        + usage.output_tokens
        + usage.cache_read_tokens
        + usage.cache_creation_tokens;
    let active_session_limit = active_session_limit(usage, Utc::now());
    let mut diagnostics = vec![
        "5h token usage is reconstructed from transcript input+output tokens".to_string(),
        "5h local estimate uses Claude Max5 token limit: 88,000".to_string(),
    ];
    let data_as_of = usage.latest_timestamp.clone();
    if data_as_of.is_none() {
        diagnostics.push("latest transcript record timestamp is unavailable".to_string());
    }
    match active_session_limit
        .as_ref()
        .map(|limit| limit.reset_source)
    {
        None => diagnostics.push("no active 5h local transcript window found".to_string()),
        Some(ResetSource::ServerAnchor) => {
            if let Some(anchor) = usage.latest_server_reset_anchor.as_ref() {
                diagnostics.push(format!(
                    "5h reset uses latest server reset anchor found in local data at {}",
                    anchor.source_path
                ));
            }
        }
        Some(ResetSource::TranscriptEstimate) => diagnostics.push(
            "5h reset is estimated from local transcript timing; official reset unavailable"
                .to_string(),
        ),
    }

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available: true,
            access_available: true,
            message: None,
            cli_authorization: None,
        },
        raw_data_available: true,
        collected_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        data_as_of,
        account: AccountInfo::default(),
        limits: active_session_limit
            .as_ref()
            .map(limit_info_from_active_session)
            .into_iter()
            .collect(),
        available_limit_resets: None,
        usage: UsageInfo {
            tokens: TokenUsage {
                input: Some(usage.input_tokens),
                cached_input: None,
                output: Some(usage.output_tokens),
                reasoning_output: None,
                cache_read: Some(usage.cache_read_tokens),
                cache_write: Some(usage.cache_creation_tokens),
                total: Some(total_tokens),
            },
            money: MoneyUsage::default(),
            activity: ActivityUsage {
                events_count: None,
                files_count: Some(usage.files as u64),
                sessions_count: Some(usage.sessions.len() as u64),
                turns_count: Some(usage.turns as u64),
                latest_activity_at: usage.latest_timestamp.clone(),
            },
            models: ModelUsage {
                top_model: top_model(&usage.models).map(str::to_string),
            },
        },
        diagnostics,
    }
}

pub(super) fn limit_info_from_active_session(session: &ActiveSessionLimit) -> LimitInfo {
    let used_percent = if session.token_limit > 0 {
        (session.used_tokens as f64 / session.token_limit as f64) * 100.0
    } else {
        0.0
    };
    let remaining_amount = session.token_limit.saturating_sub(session.used_tokens);

    LimitInfo {
        name: match session.reset_source {
            ResetSource::ServerAnchor => "5h local estimate (server reset anchor)".to_string(),
            ResetSource::TranscriptEstimate => "5h local estimate (estimated reset)".to_string(),
        },
        window_label: Some("5h".to_string()),
        window_minutes: Some(CLAUDE_LOCAL_SESSION_WINDOW_MINUTES),
        resets_at: Some(
            session
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ),
        used_percent: Some((used_percent * 10.0).round() / 10.0),
        remaining_percent: Some(((100.0 - used_percent).clamp(0.0, 100.0) * 10.0).round() / 10.0),
        used_amount: Some(session.used_tokens as f64),
        remaining_amount: Some(remaining_amount as f64),
        total_amount: Some(session.token_limit as f64),
        amount_unit: Some("tokens".to_string()),
    }
}

fn top_model(models: &HashMap<String, u64>) -> Option<&str> {
    models
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(model, _)| model.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::claude_local::model::{sample_usage, ServerResetAnchor};
    use crate::providers::claude_local::parse::parse_timestamp;

    #[test]
    fn builds_structured_data_from_representative_usage_sample() {
        let usage = sample_usage();
        let structured = structured_from_usage(&usage);

        assert_eq!(structured.provider, "claude");
        assert_eq!(structured.source, "claude_local");
        assert_eq!(structured.source_link, "docs/get-limits");
        assert!(structured.status.data_available);
        assert!(structured.status.access_available);
        assert!(structured.raw_data_available);
        assert_eq!(structured.usage.tokens.input, Some(100));
        assert_eq!(structured.usage.tokens.output, Some(40));
        assert_eq!(structured.usage.tokens.cache_read, Some(10));
        assert_eq!(structured.usage.tokens.cache_write, Some(5));
        assert_eq!(structured.usage.tokens.total, Some(155));
        assert_eq!(structured.usage.activity.files_count, Some(2));
        assert_eq!(structured.usage.activity.sessions_count, Some(2));
        assert_eq!(structured.usage.activity.turns_count, Some(5));
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(
            structured.data_as_of.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("transcript input+output tokens")));
    }

    #[test]
    fn structured_unavailable_when_transcript_roots_are_missing() {
        let structured = structured_no_roots();

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("local transcript roots were not found")
        );
        assert!(structured.raw_data_available);
        assert!(structured.limits.is_empty());
    }

    #[test]
    fn structured_unavailable_when_no_token_usage_is_found() {
        let structured = structured_no_usage(2);

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("no token usage found in 2 local transcript root(s)")
        );
        assert!(structured.raw_data_available);
    }

    #[test]
    fn raw_payload_contains_scanned_roots_and_extracted_usage() {
        let candidate_roots = vec![PathBuf::from("/tmp/.config/claude/projects")];
        let scanned_roots = candidate_roots.clone();
        let usage = sample_usage();

        let raw = encode_raw(&candidate_roots, &scanned_roots, Some(&usage)).expect("encode raw");
        let payload: Value = serde_json::from_str(&raw).expect("parse raw json");

        assert_eq!(
            payload["candidate_roots"][0].as_str(),
            Some("/tmp/.config/claude/projects")
        );
        assert_eq!(payload["usage"]["turns"].as_u64(), Some(5));
        assert_eq!(payload["usage"]["total_tokens"].as_u64(), Some(155));
        assert_eq!(
            payload["usage"]["latest_timestamp"].as_str(),
            Some("2026-06-28T10:01:00Z")
        );
        assert!(payload["usage"]["latest_server_reset_anchor"].is_null());
    }

    #[test]
    fn raw_payload_exposes_latest_server_reset_anchor_for_diagnostics() {
        let candidate_roots = vec![PathBuf::from("/tmp/.config/claude/projects")];
        let scanned_roots = candidate_roots.clone();
        let mut usage = sample_usage();
        usage.latest_server_reset_anchor = Some(ServerResetAnchor {
            resets_at: parse_timestamp("2026-06-29T08:30:00Z").expect("parse anchor"),
            source_path: "/payload/error/usage_limit/reset_time".to_string(),
        });

        let raw = encode_raw(&candidate_roots, &scanned_roots, Some(&usage)).expect("encode raw");
        let payload: Value = serde_json::from_str(&raw).expect("parse raw json");

        assert_eq!(
            payload["usage"]["latest_server_reset_anchor"]["resets_at"].as_str(),
            Some("2026-06-29T08:30:00Z")
        );
        assert_eq!(
            payload["usage"]["latest_server_reset_anchor"]["source_path"].as_str(),
            Some("/payload/error/usage_limit/reset_time")
        );
    }
}
