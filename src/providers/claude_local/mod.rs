//! Claude local transcript provider.
//!
//! Module boundaries:
//! - [`io`] — root discovery, recursive JSONL scan, and reads of the local state files
//! - [`parse`] — turn usage and server reset anchors from JSON records, profile and cached-limit parsing
//! - [`model`] — accumulated usage and 5h session-limit math
//! - [`project`] — raw JSON and structured `SourceData` projection
//! - this facade — thin [`collect`] orchestration only

mod io;
mod model;
mod parse;
mod project;

use chrono::Utc;

use crate::types::SourceData;

use self::io::{default_roots, read_profile, read_stats_cache, scan_root};
use self::model::ClaudeLocalUsage;
use self::project::{
    encode_raw, structured_from_sources, structured_no_roots, structured_no_usage,
};

pub fn collect() -> std::io::Result<SourceData> {
    let now = Utc::now();
    let candidate_roots = default_roots()?;
    let scanned_roots = candidate_roots
        .iter()
        .filter(|root| root.is_dir())
        .cloned()
        .collect::<Vec<_>>();

    let mut usage = ClaudeLocalUsage::default();
    for root in &scanned_roots {
        scan_root(root, &mut usage)?;
    }

    let profile = read_profile(now);
    let stats = read_stats_cache(now);
    let raw = encode_raw(
        &candidate_roots,
        &scanned_roots,
        Some(&usage),
        &profile,
        &stats,
    )?;
    let state_files_have_data = profile.has_data() || stats.has_data();

    if scanned_roots.is_empty() && !state_files_have_data {
        return Ok(SourceData {
            raw: Some(raw),
            structured: structured_no_roots(),
            stderr: String::new(),
        });
    }

    if usage.turns == 0 && !state_files_have_data {
        return Ok(SourceData {
            raw: Some(raw),
            structured: structured_no_usage(scanned_roots.len()),
            stderr: String::new(),
        });
    }

    Ok(SourceData {
        raw: Some(raw),
        structured: structured_from_sources(&usage, &profile, &stats, now),
        stderr: String::new(),
    })
}

#[cfg(test)]
mod tests {
    /// The macOS Keychain is not a source for Claude: its only useful field is
    /// already in `~/.claude.json`, a read can raise an interactive GUI prompt
    /// in a headless run, and the `-g` form prints the secret to stderr.
    #[test]
    fn the_source_never_reaches_for_the_macos_keychain() {
        let module_files = [
            include_str!("io.rs"),
            include_str!("model.rs"),
            include_str!("parse.rs"),
            include_str!("project.rs"),
        ];

        for content in module_files {
            assert!(!content.contains("find-generic-password"));
            assert!(!content.contains("security"));
            assert!(!content.contains("Keychain"));
        }
    }
}
