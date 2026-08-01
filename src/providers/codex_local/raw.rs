use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalRaw {
    pub root: String,
    pub files_scanned: u64,
    pub token_events: u64,
    pub totals: CodexLocalTokenTotals,
    pub latest_timestamp: Option<String>,
    pub latest_rate_limits_timestamp: Option<String>,
    pub latest_rate_limits: Option<CodexLocalRateLimits>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalTokenTotals {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
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

#[derive(Default)]
pub(super) struct CodexLocalUsage {
    pub(super) files_scanned: u64,
    pub(super) token_events: u64,
    pub(super) totals: CodexLocalTokenTotals,
    pub(super) latest_timestamp: Option<String>,
    pub(super) latest_rate_limits_timestamp: Option<String>,
    pub(super) latest_rate_limits: Option<CodexLocalRateLimits>,
}

pub(super) struct TokenEvent {
    pub(super) timestamp: Option<String>,
    pub(super) usage: Option<CodexLocalTokenTotals>,
    pub(super) rate_limits: Option<CodexLocalRateLimits>,
}

pub(super) fn raw_from_usage(root: &Path, usage: &CodexLocalUsage) -> CodexLocalRaw {
    CodexLocalRaw {
        root: root.display().to_string(),
        files_scanned: usage.files_scanned,
        token_events: usage.token_events,
        totals: usage.totals.clone(),
        latest_timestamp: usage.latest_timestamp.clone(),
        latest_rate_limits_timestamp: usage.latest_rate_limits_timestamp.clone(),
        latest_rate_limits: usage.latest_rate_limits.clone(),
    }
}
