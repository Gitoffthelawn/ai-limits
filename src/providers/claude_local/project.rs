use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};

use crate::infra::os_access::display_path;
use crate::types::{
    AccountInfo, ActivityUsage, LimitInfo, ModelUsage, MoneyUsage, SourceStatus,
    StructuredSourceInfo, TokenUsage, UsageInfo,
};

use super::model::{
    active_session_limit, ActiveSessionLimit, ClaudeLocalUsage, ResetSource,
    CLAUDE_LOCAL_SESSION_WINDOW_MINUTES,
};
use super::parse::{
    format_timestamp, CachedUsageSnapshot, CachedWindow, ClaudeProfile, ClaudeStatsCache,
};

const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_local";
const SOURCE_LINK: &str = "docs/get-limits";
const CACHED_RESET_EXPIRY_GRACE_MINUTES: i64 = 2;

pub(super) fn encode_raw(
    candidate_roots: &[PathBuf],
    scanned_roots: &[PathBuf],
    usage: Option<&ClaudeLocalUsage>,
    profile: &ClaudeProfile,
    stats: &ClaudeStatsCache,
) -> io::Result<String> {
    let mut payload = json!({
        "candidate_roots": path_strings(candidate_roots),
        "scanned_roots": path_strings(scanned_roots),
        "profile": raw_profile(profile),
        "stats_cache": raw_stats_cache(stats),
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

/// Raw data is built from the parsed and validated state-file values only. No
/// member of `~/.claude.json` outside the three documented ones is ever copied
/// into it.
fn raw_profile(profile: &ClaudeProfile) -> Value {
    json!({
        "plan": profile.plan,
        "subscription_started_at": profile.subscription_started_at,
        "cached_usage": profile.cached_usage.as_ref().map(|snapshot| json!({
            "fetched_at": format_timestamp(snapshot.fetched_at),
            "windows": snapshot.windows.iter().map(|window| json!({
                "name": window.name,
                "window_minutes": window.window_minutes,
                "resets_at": window.resets_at.map(format_timestamp),
                "used_percent": window.used_percent,
                "used_amount": window.used_amount,
                "remaining_amount": window.remaining_amount,
                "total_amount": window.total_amount,
            })).collect::<Vec<_>>(),
            "credits_total": snapshot.credits_total,
            "credits_used": snapshot.credits_used,
            "credits_remaining": snapshot.credits_remaining,
            "money_used": snapshot.money_used,
            "money_total": snapshot.money_total,
            "money_remaining": snapshot.money_remaining,
            "money_currency": snapshot.money_currency,
        })),
    })
}

fn raw_stats_cache(stats: &ClaudeStatsCache) -> Value {
    json!({
        "sessions_count": stats.sessions_count,
        "turns_count": stats.turns_count,
        "input_tokens": stats.input_tokens,
        "output_tokens": stats.output_tokens,
        "cache_read_tokens": stats.cache_read_tokens,
        "cache_creation_tokens": stats.cache_creation_tokens,
        "total_tokens": stats.total_tokens,
        "top_model": stats.top_model,
        "computed_age": stats.computed_age,
    })
}

fn path_strings(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|path| display_path(path)).collect()
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

pub(super) fn structured_from_sources(
    usage: &ClaudeLocalUsage,
    profile: &ClaudeProfile,
    stats: &ClaudeStatsCache,
    now: DateTime<Utc>,
) -> StructuredSourceInfo {
    let mut diagnostics = profile.diagnostics.clone();
    diagnostics.extend(stats.diagnostics.iter().cloned());

    let cached = profile.cached_usage.as_ref();
    let (limits, limit_diagnostics) = limits_for(usage, cached, now);
    diagnostics.extend(limit_diagnostics);

    // Every field derived from the cached `/usage` snapshot is reported with
    // the snapshot's own fetch time: it refreshes only when `/usage` is opened
    // in the TUI and must never be presented as a collection-time value.
    let data_as_of = match cached {
        Some(snapshot) => Some(format_timestamp(snapshot.fetched_at)),
        None => usage.latest_timestamp.clone(),
    };
    if data_as_of.is_none() {
        diagnostics.push("latest transcript record timestamp is unavailable".to_string());
    }

    let (usage_info, usage_diagnostics) = usage_for(usage, stats, cached);
    diagnostics.extend(usage_diagnostics);

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
        collected_at: Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        data_as_of,
        account: AccountInfo {
            plan: profile.plan.clone(),
            credits_total: cached.and_then(|snapshot| snapshot.credits_total),
            credits_used: cached.and_then(|snapshot| snapshot.credits_used),
            credits_remaining: cached.and_then(|snapshot| snapshot.credits_remaining),
            subscription_started_at: profile.subscription_started_at.clone(),
            ..AccountInfo::default()
        },
        limits,
        available_limit_resets: None,
        usage: usage_info,
        diagnostics,
    }
}

/// A usable cached snapshot is the source's limit data: it carries
/// server-computed values with real reset times. The transcript reconstruction
/// is the fallback for when the cache is absent, unusable, or stale.
fn limits_for(
    usage: &ClaudeLocalUsage,
    cached: Option<&CachedUsageSnapshot>,
    now: DateTime<Utc>,
) -> (Vec<LimitInfo>, Vec<String>) {
    let mut diagnostics = Vec::new();

    if let Some(snapshot) = cached.filter(|snapshot| !snapshot.windows.is_empty()) {
        if cached_windows_are_current(snapshot, now) {
            diagnostics.push("limits come from the cached /usage snapshot in ~/.claude.json, not from the local transcript reconstruction".to_string());
            let limits = snapshot
                .windows
                .iter()
                .map(limit_info_from_cached)
                .collect();
            return (limits, diagnostics);
        }

        diagnostics.push("cached /usage snapshot is not used for limits because an automatic reset time is already in the past".to_string());
    }

    let active = active_session_limit(usage, now);
    diagnostics
        .push("5h token usage is reconstructed from transcript input+output tokens".to_string());
    diagnostics.push("5h local estimate uses Claude Max5 token limit: 88,000".to_string());

    match active.as_ref().map(|limit| limit.reset_source) {
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

    let limits = active
        .as_ref()
        .map(limit_info_from_active_session)
        .into_iter()
        .collect();
    (limits, diagnostics)
}

/// All percentages in a snapshot were captured together, so one expired reset
/// time rejects the whole snapshot as a current-limit source.
fn cached_windows_are_current(snapshot: &CachedUsageSnapshot, now: DateTime<Utc>) -> bool {
    let cutoff = now - Duration::minutes(CACHED_RESET_EXPIRY_GRACE_MINUTES);

    !snapshot
        .windows
        .iter()
        .any(|window| window.resets_at.is_some_and(|resets_at| resets_at < cutoff))
}

/// The transcript scan stays authoritative for tokens and activity counts; the
/// aggregate cache fills a field only where the scan produced no value.
fn usage_for(
    usage: &ClaudeLocalUsage,
    stats: &ClaudeStatsCache,
    cached: Option<&CachedUsageSnapshot>,
) -> (UsageInfo, Vec<String>) {
    let scanned = usage.turns > 0;
    let mut diagnostics = Vec::new();
    let total_tokens = usage.input_tokens
        + usage.output_tokens
        + usage.cache_read_tokens
        + usage.cache_creation_tokens;

    let tokens = if scanned {
        TokenUsage {
            input: Some(usage.input_tokens),
            output: Some(usage.output_tokens),
            cache_read: Some(usage.cache_read_tokens),
            cache_write: Some(usage.cache_creation_tokens),
            total: Some(total_tokens),
            ..TokenUsage::default()
        }
    } else {
        if stats.total_tokens.is_some() {
            diagnostics.push(stats.note("token totals"));
        }
        TokenUsage {
            input: stats.input_tokens,
            output: stats.output_tokens,
            cache_read: stats.cache_read_tokens,
            cache_write: stats.cache_creation_tokens,
            total: stats.total_tokens,
            ..TokenUsage::default()
        }
    };

    let sessions_count = match usage.sessions.len() {
        0 => stats.sessions_count,
        count => Some(count as u64),
    };
    let turns_count = if scanned {
        Some(usage.turns as u64)
    } else {
        stats.turns_count
    };
    if !scanned && (sessions_count.is_some() || turns_count.is_some()) {
        diagnostics.push(stats.note("activity counts"));
    }

    let scanned_top_model = top_model(&usage.models).map(str::to_string);
    if scanned_top_model.is_none() && stats.top_model.is_some() {
        diagnostics.push(stats.note("the top model"));
    }

    let info = UsageInfo {
        tokens,
        money: MoneyUsage {
            used_amount: cached.and_then(|snapshot| snapshot.money_used),
            remaining_amount: cached.and_then(|snapshot| snapshot.money_remaining),
            total_amount: cached.and_then(|snapshot| snapshot.money_total),
            currency: cached.and_then(|snapshot| snapshot.money_currency.clone()),
        },
        activity: ActivityUsage {
            events_count: None,
            // `usage.files` counts scanned transcript files, which is a scan metric and not a
            // count of changed user files. Claude records no changed-file data, so the field
            // stays null; the scanned count remains available in raw data.
            files_count: None,
            sessions_count,
            turns_count,
            latest_activity_at: usage.latest_timestamp.clone(),
        },
        models: ModelUsage {
            top_model: scanned_top_model.or_else(|| stats.top_model.clone()),
        },
    };
    (info, diagnostics)
}

fn limit_info_from_cached(window: &CachedWindow) -> LimitInfo {
    let amounts_present = window.used_amount.is_some()
        || window.remaining_amount.is_some()
        || window.total_amount.is_some();

    LimitInfo {
        name: window.name.to_string(),
        window_label: Some(window.name.to_string()),
        window_minutes: Some(window.window_minutes),
        resets_at: window.resets_at.map(format_timestamp),
        used_percent: window.used_percent,
        remaining_percent: window
            .used_percent
            .map(|used| (100.0 - used).clamp(0.0, 100.0)),
        used_amount: window.used_amount,
        remaining_amount: window.remaining_amount,
        total_amount: window.total_amount,
        amount_unit: amounts_present.then(|| "usd".to_string()),
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

#[cfg(test)]
pub(super) fn structured_from_usage(usage: &ClaudeLocalUsage) -> StructuredSourceInfo {
    structured_from_sources(
        usage,
        &ClaudeProfile::default(),
        &ClaudeStatsCache::default(),
        Utc::now(),
    )
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
    use crate::providers::claude_local::parse::{
        parse_profile, parse_stats_cache, parse_timestamp,
    };

    /// Recognisable stand-ins for the personal data `~/.claude.json` and
    /// `~/.claude/stats-cache.json` carry. None of them may appear in any
    /// output artifact.
    const MARKERS: [&str; 12] = [
        "MARKEREMAIL",
        "MARKERACCOUNTUUID",
        "MARKERORGUUID",
        "MARKERORGNAME",
        "MARKERDISPLAYNAME",
        "MARKERREFERRAL",
        "MARKERPROJECTPATH",
        "MARKERUSERID",
        "MARKERROLE",
        "MARKERDISCLAIMER",
        "MARKERMODEL",
        "MARKERDAY",
    ];

    fn now() -> DateTime<Utc> {
        parse_timestamp("2026-08-03T20:00:00Z").expect("valid timestamp")
    }

    fn profile_json() -> String {
        r#"{
          "userID": "MARKERUSERID",
          "oauthAccount": {
            "emailAddress": "MARKEREMAIL@example.com",
            "accountUuid": "MARKERACCOUNTUUID",
            "organizationUuid": "MARKERORGUUID",
            "organizationName": "MARKERORGNAME",
            "displayName": "MARKERDISPLAYNAME",
            "organizationRole": "MARKERROLE",
            "billingType": "MARKERROLE",
            "seatTier": "MARKERROLE",
            "organizationRateLimitTier": "MARKERROLE",
            "userRateLimitTier": "MARKERROLE",
            "organizationType": "claude_pro",
            "subscriptionCreatedAt": "2026-06-06T18:13:09.971910Z",
            "profileFetchedAt": 1785700000000
          },
          "passesEligibilityCache": {
            "referral_code_details": { "referral_code": "MARKERREFERRAL" }
          },
          "projects": { "/Users/MARKERPROJECTPATH/work": { "allowedTools": [] } },
          "cachedUsageUtilization": {
            "accountUuid": "MARKERACCOUNTUUID",
            "fetchedAtMs": 1785782400000,
            "utilization": {
              "five_hour": {
                "utilization": 92,
                "resets_at": "2026-08-03T22:30:00.652763+00:00",
                "limit_dollars": null,
                "used_dollars": null,
                "remaining_dollars": null
              },
              "seven_day": {
                "utilization": 30,
                "resets_at": "2026-08-04T10:00:00.652788+00:00",
                "limit_dollars": null,
                "used_dollars": null,
                "remaining_dollars": null
              },
              "tangelo": { "utilization": 12, "resets_at": "2026-08-04T10:00:00Z" },
              "limits": [
                {
                  "kind": "session",
                  "percent": 92,
                  "severity": "critical",
                  "resets_at": "2026-08-03T22:30:00.652763+00:00",
                  "is_active": true
                },
                {
                  "kind": "weekly_all",
                  "percent": 30,
                  "severity": "normal",
                  "resets_at": "2026-08-04T10:00:00.652788+00:00",
                  "is_active": false
                }
              ],
              "extra_usage": {
                "is_enabled": true,
                "monthly_limit": 5000,
                "used_credits": 1081,
                "currency": "EUR",
                "decimal_places": 2
              },
              "spend": {
                "used": { "amount_minor": 1081, "currency": "EUR", "exponent": 2 },
                "limit": 5000,
                "disclaimer": "MARKERDISCLAIMER"
              }
            }
          }
        }"#
        .to_string()
    }

    fn stats_cache_json() -> String {
        r#"{
          "totalSessions": 284,
          "totalMessages": 52843,
          "lastComputedDate": "2026-08-02",
          "firstSessionDate": "MARKERDAY",
          "longestSession": { "path": "/Users/MARKERPROJECTPATH/work" },
          "dailyActivity": { "MARKERDAY": 7 },
          "modelUsage": {
            "claude-sonnet-4-6": {
              "inputTokens": 100,
              "outputTokens": 20,
              "cacheReadInputTokens": 3000,
              "cacheCreationInputTokens": 400,
              "costUSD": 12.5
            },
            "MARKERMODEL /Users/MARKERPROJECTPATH": {
              "inputTokens": 9000000,
              "outputTokens": 2,
              "cacheReadInputTokens": 3,
              "cacheCreationInputTokens": 4
            }
          }
        }"#
        .to_string()
    }

    fn assert_free_of_markers(label: &str, text: &str) {
        for marker in MARKERS {
            assert!(
                !text.contains(marker),
                "{label} leaked the personal-data marker {marker}"
            );
        }
    }

    #[test]
    fn personal_data_from_local_state_files_never_reaches_any_output_artifact() {
        let profile = parse_profile(&profile_json(), now());
        let stats = parse_stats_cache(&stats_cache_json(), now());
        let usage = sample_usage();
        let structured = structured_from_sources(&usage, &profile, &stats, now());
        let raw = encode_raw(
            &[PathBuf::from("/tmp/.claude/projects")],
            &[PathBuf::from("/tmp/.claude/projects")],
            Some(&usage),
            &profile,
            &stats,
        )
        .expect("encode raw");

        // The read must have produced real data, or the assertions below are vacuous.
        assert_eq!(structured.account.plan.as_deref(), Some("claude_pro"));
        assert_eq!(structured.limits.len(), 2);
        assert!(!structured.diagnostics.is_empty());

        assert_free_of_markers("raw data", &raw);
        assert_free_of_markers(
            "structured data",
            &serde_json::to_string(&structured).expect("serialize structured"),
        );
        assert_free_of_markers(
            "status message",
            structured.status.message.as_deref().unwrap_or_default(),
        );
        for diagnostic in &structured.diagnostics {
            assert_free_of_markers("a diagnostic", diagnostic);
        }
        for diagnostic in profile.diagnostics.iter().chain(stats.diagnostics.iter()) {
            assert_free_of_markers("a parser diagnostic", diagnostic);
        }
    }

    #[test]
    fn an_unusable_plan_value_is_dropped_instead_of_being_reported() {
        let content = profile_json().replace("\"claude_pro\"", "\"MARKERORGNAME plan\"");
        let profile = parse_profile(&content, now());
        let structured = structured_from_sources(
            &sample_usage(),
            &profile,
            &ClaudeStatsCache::default(),
            now(),
        );

        assert!(structured.account.plan.is_none());
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry == "plan: the local profile value is not a usable plan name"));
        assert_free_of_markers(
            "structured data",
            &serde_json::to_string(&structured).expect("serialize structured"),
        );
    }

    #[test]
    fn cached_snapshot_fetch_time_becomes_data_as_of() {
        let profile = parse_profile(&profile_json(), now());
        let structured = structured_from_sources(
            &sample_usage(),
            &profile,
            &ClaudeStatsCache::default(),
            now(),
        );

        assert_eq!(
            structured.data_as_of.as_deref(),
            Some("2026-08-03T18:40:00Z")
        );
        assert_ne!(
            structured.data_as_of.as_deref(),
            structured.usage.activity.latest_activity_at.as_deref()
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("refreshed only when /usage is opened")));
    }

    #[test]
    fn cached_snapshot_supplies_limits_credits_and_spend() {
        let profile = parse_profile(&profile_json(), now());
        let structured = structured_from_sources(
            &sample_usage(),
            &profile,
            &ClaudeStatsCache::default(),
            now(),
        );

        let five_hour = &structured.limits[0];
        assert_eq!(five_hour.name, "five_hour");
        assert_eq!(five_hour.window_minutes, Some(300));
        assert_eq!(five_hour.used_percent, Some(92.0));
        assert_eq!(five_hour.remaining_percent, Some(8.0));
        assert_eq!(five_hour.resets_at.as_deref(), Some("2026-08-03T22:30:00Z"));
        assert_eq!(structured.limits[1].name, "seven_day");
        assert_eq!(structured.limits[1].window_minutes, Some(10_080));

        assert_eq!(structured.account.credits_total, Some(50.0));
        assert_eq!(structured.account.credits_used, Some(10.81));
        assert_eq!(structured.account.credits_remaining, Some(39.19));
        assert_eq!(
            structured.account.subscription_started_at.as_deref(),
            Some("2026-06-06T18:13:09Z")
        );
        assert_eq!(structured.usage.money.used_amount, Some(10.81));
        assert_eq!(structured.usage.money.total_amount, Some(50.0));
        assert_eq!(structured.usage.money.currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn code_named_windows_are_not_projected() {
        let profile = parse_profile(&profile_json(), now());
        let structured = structured_from_sources(
            &sample_usage(),
            &profile,
            &ClaudeStatsCache::default(),
            now(),
        );

        assert!(!structured
            .limits
            .iter()
            .any(|limit| limit.name == "tangelo"));
    }

    #[test]
    fn expired_cached_snapshot_falls_back_to_the_transcript_reconstruction() {
        let profile = parse_profile(&profile_json(), now());
        let late = parse_timestamp("2026-08-05T00:00:00Z").expect("valid timestamp");
        let structured = structured_from_sources(
            &sample_usage(),
            &profile,
            &ClaudeStatsCache::default(),
            late,
        );

        assert!(!structured
            .limits
            .iter()
            .any(|limit| limit.name == "five_hour"));
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("automatic reset time is already in the past")));
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("Claude Max5 token limit")));
    }

    #[test]
    fn transcript_scan_stays_authoritative_over_the_aggregate_cache() {
        let stats = parse_stats_cache(&stats_cache_json(), now());
        let structured =
            structured_from_sources(&sample_usage(), &ClaudeProfile::default(), &stats, now());

        assert_eq!(structured.usage.tokens.input, Some(100));
        assert_eq!(structured.usage.tokens.total, Some(155));
        assert_eq!(structured.usage.activity.sessions_count, Some(2));
        assert_eq!(structured.usage.activity.turns_count, Some(5));
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
        assert!(!structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("stats-cache.json")));
    }

    #[test]
    fn aggregate_cache_fills_fields_the_transcript_scan_did_not_produce() {
        let stats = parse_stats_cache(&stats_cache_json(), now());
        let structured = structured_from_sources(
            &ClaudeLocalUsage::default(),
            &ClaudeProfile::default(),
            &stats,
            now(),
        );

        assert_eq!(structured.usage.tokens.input, Some(9_000_100));
        assert_eq!(structured.usage.tokens.output, Some(22));
        assert_eq!(structured.usage.tokens.cache_read, Some(3003));
        assert_eq!(structured.usage.tokens.cache_write, Some(404));
        assert_eq!(structured.usage.tokens.total, Some(9_003_529));
        assert_eq!(structured.usage.activity.sessions_count, Some(284));
        assert_eq!(structured.usage.activity.turns_count, Some(52_843));
        assert_eq!(structured.usage.activity.files_count, None);
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("stats-cache.json") && entry.contains("1d ago")));
        // The largest aggregate belongs to an unusable model name and is
        // rejected instead of being reported as the top model.
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("not a usable model name")));
    }

    #[test]
    fn missing_profile_file_degrades_to_the_transcript_reconstruction() {
        let profile = ClaudeProfile::failed("local profile: ~/.claude.json was not found");
        let structured = structured_from_sources(
            &sample_usage(),
            &profile,
            &ClaudeStatsCache::default(),
            now(),
        );

        assert!(structured.status.data_available);
        assert!(structured.account.plan.is_none());
        assert!(structured.account.subscription_started_at.is_none());
        assert!(structured.account.credits_used.is_none());
        assert!(structured.usage.money.used_amount.is_none());
        assert_eq!(
            structured.data_as_of.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry == "local profile: ~/.claude.json was not found"));
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("Claude Max5 token limit")));
    }

    #[test]
    fn unparseable_state_files_never_carry_their_content_into_diagnostics() {
        let profile = parse_profile("{ not json MARKEREMAIL", now());
        let stats = parse_stats_cache("{ not json MARKERMODEL", now());

        assert_eq!(
            profile.diagnostics,
            vec!["local profile: ~/.claude.json could not be parsed".to_string()]
        );
        assert_eq!(
            stats.diagnostics,
            vec!["usage aggregates: ~/.claude/stats-cache.json could not be parsed".to_string()]
        );
        assert!(!profile.has_data());
        assert!(!stats.has_data());
    }

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
        // Scanned transcript files are a scan metric, never a changed-file count.
        assert_eq!(structured.usage.activity.files_count, None);
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

        let raw = encode_raw(
            &candidate_roots,
            &scanned_roots,
            Some(&usage),
            &ClaudeProfile::default(),
            &ClaudeStatsCache::default(),
        )
        .expect("encode raw");
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

    /// Every default transcript root sits in the home directory, so the raw
    /// payload would otherwise spell out the account name.
    #[test]
    fn raw_payload_shortens_the_home_directory_in_the_transcript_roots() {
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
            return;
        };
        let candidate_roots =
            crate::infra::os_access::claude_local_roots().expect("HOME should be available");

        let raw = encode_raw(
            &candidate_roots,
            &candidate_roots,
            Some(&sample_usage()),
            &ClaudeProfile::default(),
            &ClaudeStatsCache::default(),
        )
        .expect("encode raw");
        let payload: Value = serde_json::from_str(&raw).expect("parse raw json");

        assert_eq!(
            payload["candidate_roots"][0].as_str(),
            Some("~/.config/claude/projects")
        );
        assert!(
            !raw.contains(&home.display().to_string()),
            "the home directory leaked into the raw payload"
        );
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

        let raw = encode_raw(
            &candidate_roots,
            &scanned_roots,
            Some(&usage),
            &ClaudeProfile::default(),
            &ClaudeStatsCache::default(),
        )
        .expect("encode raw");
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
