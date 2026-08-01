use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::infra::os_access::claude_local_roots;

use super::model::ClaudeLocalUsage;
use super::parse::{extract_server_reset_anchor, extract_turn_usage};

pub(super) fn default_roots() -> io::Result<Vec<PathBuf>> {
    claude_local_roots()
}

pub(super) fn scan_root(root: &Path, usage: &mut ClaudeLocalUsage) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_root(&path, usage)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            scan_jsonl_file(&path, usage)?;
        }
    }

    Ok(())
}

pub(super) fn scan_jsonl_file(path: &Path, usage: &mut ClaudeLocalUsage) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut seen_messages = HashMap::<String, super::model::TurnUsage>::new();
    let mut turns_without_id = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let Ok(record) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        if let Some(anchor) = extract_server_reset_anchor(&record) {
            if usage
                .latest_server_reset_anchor
                .as_ref()
                .is_none_or(|current| anchor > *current)
            {
                usage.latest_server_reset_anchor = Some(anchor);
            }
        }

        let Some(turn) = extract_turn_usage(&record) else {
            continue;
        };

        if let Some(message_id) = turn.message_id.clone().filter(|value| !value.is_empty()) {
            seen_messages.insert(message_id, turn);
        } else {
            turns_without_id.push(turn);
        }
    }

    let turn_count = seen_messages.len() + turns_without_id.len();
    if turn_count > 0 {
        usage.files += 1;
    }

    for turn in seen_messages.into_values().chain(turns_without_id) {
        usage.sessions.insert(turn.session_id.clone());
        usage.turns += 1;
        usage.input_tokens += turn.input_tokens;
        usage.output_tokens += turn.output_tokens;
        usage.cache_read_tokens += turn.cache_read_tokens;
        usage.cache_creation_tokens += turn.cache_creation_tokens;

        if let Some(model) = turn.model.as_ref().filter(|value| !value.is_empty()) {
            *usage.models.entry(model.clone()).or_default() += 1;
        }

        if let Some(timestamp) = turn.timestamp.as_ref().filter(|value| !value.is_empty()) {
            if usage
                .latest_timestamp
                .as_ref()
                .is_none_or(|current| timestamp > current)
            {
                usage.latest_timestamp = Some(timestamp.clone());
            }
        }

        usage.turns_by_time.push(turn);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;

    use super::*;
    use crate::providers::claude_local::project::structured_from_usage;

    #[test]
    fn scans_usage_and_deduplicates_streaming_message_records() {
        let path = env::temp_dir().join(format!(
            "ai-limits-claude-local-{}.jsonl",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{"type":"assistant","sessionId":"s1","timestamp":"2026-06-28T10:00:00Z","message":{"id":"m1","model":"claude-sonnet-4-6","usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":1,"cache_creation_input_tokens":2}}}
{"type":"assistant","sessionId":"s1","timestamp":"2026-06-28T10:01:00Z","message":{"id":"m1","model":"claude-sonnet-4-6","usage":{"input_tokens":30,"output_tokens":7,"cache_read_input_tokens":3,"cache_creation_input_tokens":4}}}
{"type":"assistant","sessionId":"s2","timestamp":"2026-06-28T10:02:00Z","message":{"model":"claude-haiku-4-5","usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}
"#,
        )
        .expect("write fixture");

        let mut usage = ClaudeLocalUsage::default();
        scan_jsonl_file(&path, &mut usage).expect("scan fixture");
        let structured = structured_from_usage(&usage);
        let _ = fs::remove_file(&path);

        assert_eq!(usage.files, 1);
        assert_eq!(usage.sessions.len(), 1);
        assert_eq!(usage.turns, 1);
        assert_eq!(usage.input_tokens, 30);
        assert_eq!(usage.output_tokens, 7);
        assert_eq!(usage.cache_read_tokens, 3);
        assert_eq!(usage.cache_creation_tokens, 4);

        assert!(structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(structured.usage.tokens.input, Some(30));
        assert_eq!(structured.usage.tokens.output, Some(7));
        assert_eq!(structured.usage.tokens.cache_read, Some(3));
        assert_eq!(structured.usage.tokens.cache_write, Some(4));
        assert_eq!(structured.usage.tokens.total, Some(44));
        assert_eq!(structured.usage.activity.turns_count, Some(1));
        assert_eq!(
            structured.usage.activity.latest_activity_at.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
    }
}
