//! Claude local transcript provider.
//!
//! Module boundaries:
//! - [`io`] — root discovery and recursive JSONL scan
//! - [`parse`] — turn usage and server reset anchors from JSON records
//! - [`model`] — accumulated usage and 5h session-limit math
//! - [`project`] — raw JSON and structured `SourceData` projection
//! - this facade — thin [`collect`] orchestration only

mod io;
mod model;
mod parse;
mod project;

use crate::types::SourceData;

use self::io::{default_roots, scan_root};
use self::model::ClaudeLocalUsage;
use self::project::{encode_raw, structured_from_usage, structured_no_roots, structured_no_usage};

pub fn collect() -> std::io::Result<SourceData> {
    let candidate_roots = default_roots()?;
    let scanned_roots = candidate_roots
        .iter()
        .filter(|root| root.is_dir())
        .cloned()
        .collect::<Vec<_>>();

    if scanned_roots.is_empty() {
        return Ok(SourceData {
            raw: Some(encode_raw(&candidate_roots, &scanned_roots, None)?),
            structured: structured_no_roots(),
            stderr: String::new(),
        });
    }

    let mut usage = ClaudeLocalUsage::default();

    for root in &scanned_roots {
        scan_root(root, &mut usage)?;
    }

    if usage.turns == 0 {
        return Ok(SourceData {
            raw: Some(encode_raw(&candidate_roots, &scanned_roots, Some(&usage))?),
            structured: structured_no_usage(scanned_roots.len()),
            stderr: String::new(),
        });
    }

    Ok(SourceData {
        raw: Some(encode_raw(&candidate_roots, &scanned_roots, Some(&usage))?),
        structured: structured_from_usage(&usage),
        stderr: String::new(),
    })
}
