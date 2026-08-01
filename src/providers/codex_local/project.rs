use chrono::{DateTime, Utc};

use crate::types::{
    AccountInfo, ActivityUsage, LimitInfo, SourceData, SourceStatus, StructuredSourceInfo,
    TokenUsage, UsageInfo,
};

use super::raw::{CodexLocalRateLimitWindow, CodexLocalRateLimits, CodexLocalRaw};

const PROVIDER: &str = "codex";
const SOURCE: &str = "codex_local";
const SOURCE_LINK: &str = "docs/get-limits";

pub fn decode_raw(raw: Option<&str>) -> Option<CodexLocalRaw> {
    raw.and_then(|value| serde_json::from_str(value).ok())
}

pub(super) fn source_data_from_raw(
    raw: &CodexLocalRaw,
    structured: StructuredSourceInfo,
) -> SourceData {
    SourceData {
        raw: serde_json::to_string(raw).ok(),
        structured,
        stderr: String::new(),
    }
}

pub(super) fn build_structured(
    raw: &CodexLocalRaw,
    collected_at: Option<String>,
    access_available: bool,
    data_available: bool,
    message: Option<String>,
) -> StructuredSourceInfo {
    let mut diagnostics = Vec::new();
    let mut limits = Vec::new();

    if let Some(rate_limits) = &raw.latest_rate_limits {
        if let Some(primary) = &rate_limits.primary {
            limits.push(limit_from_window("primary", primary));
        }
        if let Some(secondary) = &rate_limits.secondary {
            limits.push(limit_from_window("secondary", secondary));
        }
        if rate_limits.credits_unlimited {
            diagnostics.push("credits: unlimited".to_string());
        }
    } else if data_available {
        diagnostics.push("limits/reset: unavailable in local Codex JSONL".to_string());
    }

    let account = account_from_rate_limits(raw.latest_rate_limits.as_ref());
    let usage = usage_from_raw(raw);
    let data_as_of = raw
        .latest_rate_limits_timestamp
        .clone()
        .or_else(|| raw.latest_timestamp.clone());
    if data_available && data_as_of.is_none() {
        diagnostics.push("latest source record timestamp is unavailable".to_string());
    }

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available,
            access_available,
            message,
            cli_authorization: None,
        },
        raw_data_available: true,
        collected_at,
        data_as_of,
        account,
        limits,
        available_limit_resets: None,
        usage,
        diagnostics,
    }
}

fn account_from_rate_limits(rate_limits: Option<&CodexLocalRateLimits>) -> AccountInfo {
    let Some(rate_limits) = rate_limits else {
        return AccountInfo::default();
    };

    AccountInfo {
        plan: rate_limits.plan_type.clone(),
        credits_total: None,
        credits_used: None,
        credits_remaining: if rate_limits.credits_unlimited {
            None
        } else {
            rate_limits.credits
        },
    }
}

fn usage_from_raw(raw: &CodexLocalRaw) -> UsageInfo {
    let has_tokens = raw.token_events > 0;
    UsageInfo {
        tokens: if has_tokens {
            TokenUsage {
                input: Some(raw.totals.input_tokens),
                cached_input: Some(raw.totals.cached_input_tokens),
                output: Some(raw.totals.output_tokens),
                reasoning_output: Some(raw.totals.reasoning_output_tokens),
                cache_read: None,
                cache_write: None,
                total: Some(raw.totals.total_tokens),
            }
        } else {
            TokenUsage::default()
        },
        activity: ActivityUsage {
            events_count: Some(raw.token_events),
            files_count: Some(raw.files_scanned),
            sessions_count: None,
            turns_count: None,
            latest_activity_at: raw
                .latest_rate_limits_timestamp
                .clone()
                .or_else(|| raw.latest_timestamp.clone()),
        },
        ..UsageInfo::default()
    }
}

fn limit_from_window(name: &str, window: &CodexLocalRateLimitWindow) -> LimitInfo {
    let remaining_pct = window.used_percent.map(calc_remaining_percent);

    LimitInfo {
        name: name.to_string(),
        window_label: window.window_minutes.map(window_label),
        window_minutes: window.window_minutes,
        resets_at: window.resets_at.map(format_unix_utc),
        used_percent: window.used_percent,
        remaining_percent: remaining_pct,
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    }
}

fn window_label(minutes: u64) -> String {
    match minutes {
        300 => "5h (300m)".to_string(),
        10080 => "weekly (10080m)".to_string(),
        _ => format!("{minutes}m"),
    }
}

fn calc_remaining_percent(used_percent: f64) -> f64 {
    let raw = (100.0 - used_percent).max(0.0);
    (raw * 10.0).round() / 10.0
}

pub(super) fn format_unix_utc(seconds: u64) -> String {
    DateTime::<Utc>::from_timestamp(seconds as i64, 0)
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{seconds} (unix)"))
}
