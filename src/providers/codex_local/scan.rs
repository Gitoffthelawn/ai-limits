use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::infra::os_access::codex_local_root;

use super::parse::{parse_record, CodexLocalRecord};
use super::raw::{digest, CodexLocalUsage, TokenEvent};

pub(super) fn codex_home() -> io::Result<PathBuf> {
    codex_local_root()
}

pub(super) fn scan_dir(path: &Path, usage: &mut CodexLocalUsage) -> io::Result<()> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            scan_dir(&path, usage)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "jsonl") {
            scan_file(&path, usage)?;
        }
    }

    Ok(())
}

pub(super) fn scan_file(path: &Path, usage: &mut CodexLocalUsage) -> io::Result<()> {
    usage.files_scanned += 1;

    let content = fs::read_to_string(path)?;

    for line in content.lines() {
        match parse_record(line) {
            Some(CodexLocalRecord::TokenCount(event)) => apply_token_event(event, usage),
            Some(CodexLocalRecord::Session(session_id)) => {
                usage.session_digests.insert(digest(&session_id));
            }
            Some(CodexLocalRecord::Turn(turn_id)) => {
                usage.turn_digests.insert(digest(&turn_id));
            }
            Some(CodexLocalRecord::Changes(paths)) => {
                for path in paths {
                    usage.changed_file_digests.insert(digest(&path));
                }
            }
            None => continue,
        }
    }

    Ok(())
}

fn apply_token_event(event: TokenEvent, usage: &mut CodexLocalUsage) {
    usage.token_events += 1;

    if let Some(tokens) = &event.usage {
        usage.totals.input_tokens += tokens.input_tokens;
        usage.totals.cached_input_tokens += tokens.cached_input_tokens;
        usage.totals.output_tokens += tokens.output_tokens;
        usage.totals.reasoning_output_tokens += tokens.reasoning_output_tokens;
        usage.totals.total_tokens += tokens.total_tokens;

        if let Some(cache_write) = tokens.cache_write_input_tokens {
            usage.totals.cache_write_input_tokens =
                Some(usage.totals.cache_write_input_tokens.unwrap_or(0) + cache_write);
        }
    }

    let Some(timestamp) = event.timestamp else {
        return;
    };

    if usage
        .latest_timestamp
        .as_ref()
        .is_none_or(|latest| timestamp > *latest)
    {
        usage.latest_timestamp = Some(timestamp.clone());
    }

    if event.rate_limits.is_some()
        && usage
            .latest_rate_limits_timestamp
            .as_ref()
            .is_none_or(|latest| timestamp > *latest)
    {
        usage.latest_rate_limits_timestamp = Some(timestamp);
        usage.latest_rate_limits = event.rate_limits;
    }
}
