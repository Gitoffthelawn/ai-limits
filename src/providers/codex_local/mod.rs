mod parse;
mod project;
mod raw;
mod scan;

use chrono::Utc;

use crate::types::SourceData;

pub use project::decode_raw;
pub use raw::{
    CodexLocalRateLimitWindow, CodexLocalRateLimits, CodexLocalRaw, CodexLocalTokenTotals,
};

use project::{build_structured, source_data_from_raw};
use raw::{raw_from_usage, CodexLocalUsage};
use scan::{codex_home, scan_dir};

pub fn get_usage() -> std::io::Result<SourceData> {
    collect()
}

pub fn collect() -> std::io::Result<SourceData> {
    let root = codex_home()?;
    let collected_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());

    if !root.exists() {
        let raw = CodexLocalRaw {
            root: root.display().to_string(),
            ..CodexLocalRaw::default()
        };
        let structured = build_structured(
            &raw,
            collected_at,
            false,
            false,
            Some(format!("not found: {}", root.display())),
        );
        return Ok(source_data_from_raw(&raw, structured));
    }

    let mut usage = CodexLocalUsage::default();
    scan_dir(&root.join("sessions"), &mut usage)?;
    scan_dir(&root.join("archived_sessions"), &mut usage)?;

    let raw = raw_from_usage(&root, &usage);
    let (data_available, message) = if usage.token_events == 0 {
        (false, Some("token events: not found".to_string()))
    } else {
        (true, None)
    };
    let structured = build_structured(&raw, collected_at, true, data_available, message);

    Ok(source_data_from_raw(&raw, structured))
}

#[cfg(test)]
mod tests;
