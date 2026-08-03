use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::infra::os_access::display_path;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalRaw {
    pub root: String,
    /// Internal scan metric about the source itself, never an activity count.
    pub files_scanned: u64,
    pub token_events: u64,
    #[serde(default)]
    pub sessions_count: u64,
    #[serde(default)]
    pub turns_count: u64,
    /// Distinct user files touched by applied patches.
    #[serde(default)]
    pub changed_files_count: u64,
    pub totals: CodexLocalTokenTotals,
    pub latest_timestamp: Option<String>,
    pub latest_rate_limits_timestamp: Option<String>,
    pub latest_rate_limits: Option<CodexLocalRateLimits>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalTokenTotals {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    /// `None` while no scanned event reported the field at all.
    pub cache_write_input_tokens: Option<u64>,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalRateLimits {
    pub primary: Option<CodexLocalRateLimitWindow>,
    pub secondary: Option<CodexLocalRateLimitWindow>,
    pub credits: Option<f64>,
    pub credits_unlimited: bool,
    pub plan_type: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalRateLimitWindow {
    pub used_percent: Option<f64>,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<u64>,
}

/// Accumulation model for one scan.
///
/// Session ids, turn ids, and changed file paths are identifiers and absolute
/// paths from the user's machine, so only their digests are kept: the scan
/// needs to know how many distinct values it saw and nothing else.
#[derive(Default)]
pub(super) struct CodexLocalUsage {
    pub(super) files_scanned: u64,
    pub(super) token_events: u64,
    pub(super) session_digests: HashSet<u64>,
    pub(super) turn_digests: HashSet<u64>,
    pub(super) changed_file_digests: HashSet<u64>,
    pub(super) totals: CodexLocalTokenTotals,
    pub(super) latest_timestamp: Option<String>,
    pub(super) latest_rate_limits_timestamp: Option<String>,
    pub(super) latest_rate_limits: Option<CodexLocalRateLimits>,
}

pub(super) fn digest(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub(super) struct TokenEvent {
    pub(super) timestamp: Option<String>,
    pub(super) usage: Option<CodexLocalTokenTotals>,
    pub(super) rate_limits: Option<CodexLocalRateLimits>,
}

pub(super) fn raw_from_usage(root: &Path, usage: &CodexLocalUsage) -> CodexLocalRaw {
    CodexLocalRaw {
        root: display_path(root),
        files_scanned: usage.files_scanned,
        token_events: usage.token_events,
        sessions_count: usage.session_digests.len() as u64,
        turns_count: usage.turn_digests.len() as u64,
        changed_files_count: usage.changed_file_digests.len() as u64,
        totals: usage.totals.clone(),
        latest_timestamp: usage.latest_timestamp.clone(),
        latest_rate_limits_timestamp: usage.latest_rate_limits_timestamp.clone(),
        latest_rate_limits: usage.latest_rate_limits.clone(),
    }
}
