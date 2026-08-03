use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

use super::model::{ServerResetAnchor, TurnUsage};

/// Plan and model names are the only strings read out of the local state files.
/// They are accepted only in the short token shape a name actually has, so that
/// no other file content can travel out of this module inside them.
const PLAN_NAME_MAX_CHARS: usize = 32;
const MODEL_NAME_MAX_CHARS: usize = 64;
const MAX_DECIMAL_PLACES: i64 = 6;

const FIVE_HOUR_WINDOW_MINUTES: u64 = 5 * 60;
const SEVEN_DAY_WINDOW_MINUTES: u64 = 7 * 24 * 60;

/// The named windows projected from a cached snapshot, each with the `limits[]`
/// record kind describing the same window. Code-named windows are not parsed:
/// their meaning, length, and applicability are unknown.
const CACHED_WINDOWS: [(&str, u64, &str); 2] = [
    ("five_hour", FIVE_HOUR_WINDOW_MINUTES, "session"),
    ("seven_day", SEVEN_DAY_WINDOW_MINUTES, "weekly_all"),
];

pub(super) fn extract_turn_usage(record: &Value) -> Option<TurnUsage> {
    if record.get("type")?.as_str()? != "assistant" {
        return None;
    }

    let session_id = record.get("sessionId")?.as_str()?.to_string();
    let message = record.get("message")?;
    let usage = message.get("usage")?;
    let input_tokens = number_field(usage, "input_tokens");
    let output_tokens = number_field(usage, "output_tokens");
    let cache_read_tokens = number_field(usage, "cache_read_input_tokens");
    let cache_creation_tokens = number_field(usage, "cache_creation_input_tokens");

    if input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens == 0 {
        return None;
    }

    Some(TurnUsage {
        session_id,
        timestamp: record
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        model: message
            .get("model")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        message_id: message
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
    })
}

fn number_field(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

pub(super) fn extract_server_reset_anchor(record: &Value) -> Option<ServerResetAnchor> {
    let mut candidates = Vec::new();
    collect_server_reset_anchor_candidates(record, "", false, &mut candidates);
    candidates.into_iter().max()
}

fn collect_server_reset_anchor_candidates(
    value: &Value,
    path: &str,
    in_reset_context: bool,
    candidates: &mut Vec<ServerResetAnchor>,
) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = join_json_path(path, key);
                let key_is_reset_context = in_reset_context || is_server_reset_context_key(key);

                if is_reset_timestamp(key) && key_is_reset_context {
                    if let Some(resets_at) = parse_reset_timestamp_value(child) {
                        candidates.push(ServerResetAnchor {
                            resets_at,
                            source_path: child_path.clone(),
                        });
                    }
                }

                collect_server_reset_anchor_candidates(
                    child,
                    &child_path,
                    key_is_reset_context,
                    candidates,
                );
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                collect_server_reset_anchor_candidates(
                    child,
                    &child_path,
                    in_reset_context,
                    candidates,
                );
            }
        }
        _ => {}
    }
}

fn join_json_path(path: &str, key: &str) -> String {
    let escaped = key.replace('~', "~0").replace('/', "~1");
    if path.is_empty() {
        format!("/{escaped}")
    } else {
        format!("{path}/{escaped}")
    }
}

fn is_server_reset_context_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    normalized.contains("ratelimit")
        || normalized.contains("usagelimit")
        || normalized.contains("usage")
        || normalized.contains("quota")
        || normalized.contains("429")
}

fn is_reset_timestamp(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    matches!(
        normalized.as_str(),
        "resetsat" | "resetat" | "resettime" | "resettimestamp" | "limitresetat"
    )
}

fn parse_reset_timestamp_value(value: &Value) -> Option<DateTime<Utc>> {
    if let Some(timestamp) = value.as_u64() {
        return DateTime::from_timestamp(timestamp as i64, 0);
    }

    if let Some(timestamp) = value.as_i64() {
        return DateTime::from_timestamp(timestamp, 0);
    }

    let text = value.as_str()?.trim();
    if text.is_empty() {
        return None;
    }

    if let Ok(timestamp) = text.parse::<i64>() {
        return DateTime::from_timestamp(timestamp, 0);
    }

    parse_timestamp(text)
}

pub(super) fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .ok()
}

pub(super) fn format_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Profile facts read from `~/.claude.json`.
///
/// Only the plan name, the subscription start date, and the cached `/usage`
/// snapshot ever leave this module. Email, display name, organization name,
/// account and organization UUIDs, referral codes, project paths, and every
/// other member of the file are not read into this struct at all.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ClaudeProfile {
    pub(super) plan: Option<String>,
    pub(super) subscription_started_at: Option<String>,
    pub(super) cached_usage: Option<CachedUsageSnapshot>,
    pub(super) diagnostics: Vec<String>,
}

impl ClaudeProfile {
    pub(super) fn failed(diagnostic: &str) -> Self {
        Self {
            diagnostics: vec![diagnostic.to_string()],
            ..Self::default()
        }
    }

    pub(super) fn has_data(&self) -> bool {
        self.plan.is_some() || self.subscription_started_at.is_some() || self.cached_usage.is_some()
    }
}

/// The `/usage` snapshot the Claude TUI caches in `~/.claude.json`.
///
/// `fetched_at` is mandatory: every field derived from this cache is reported
/// with it as `data_as_of`, because the cache is refreshed only when the user
/// opens `/usage` and can otherwise be arbitrarily stale.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CachedUsageSnapshot {
    pub(super) fetched_at: DateTime<Utc>,
    pub(super) windows: Vec<CachedWindow>,
    pub(super) credits_total: Option<f64>,
    pub(super) credits_used: Option<f64>,
    pub(super) credits_remaining: Option<f64>,
    pub(super) money_used: Option<f64>,
    pub(super) money_total: Option<f64>,
    pub(super) money_remaining: Option<f64>,
    pub(super) money_currency: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct CachedWindow {
    pub(super) name: &'static str,
    pub(super) window_minutes: u64,
    pub(super) resets_at: Option<DateTime<Utc>>,
    pub(super) used_percent: Option<f64>,
    pub(super) used_amount: Option<f64>,
    pub(super) remaining_amount: Option<f64>,
    pub(super) total_amount: Option<f64>,
}

/// Usage aggregates read from `~/.claude/stats-cache.json`.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ClaudeStatsCache {
    pub(super) sessions_count: Option<u64>,
    pub(super) turns_count: Option<u64>,
    pub(super) input_tokens: Option<u64>,
    pub(super) output_tokens: Option<u64>,
    pub(super) cache_read_tokens: Option<u64>,
    pub(super) cache_creation_tokens: Option<u64>,
    pub(super) total_tokens: Option<u64>,
    pub(super) top_model: Option<String>,
    /// Age of the aggregates, already rendered as a fixed-shape phrase.
    pub(super) computed_age: Option<String>,
    pub(super) diagnostics: Vec<String>,
}

impl ClaudeStatsCache {
    pub(super) fn failed(diagnostic: &str) -> Self {
        Self {
            diagnostics: vec![diagnostic.to_string()],
            ..Self::default()
        }
    }

    pub(super) fn has_data(&self) -> bool {
        self.sessions_count.is_some() || self.turns_count.is_some() || self.total_tokens.is_some()
    }

    /// Fixed-literal sentence naming the cache and the age of its aggregates.
    pub(super) fn note(&self, subject: &str) -> String {
        match self.computed_age.as_deref() {
            Some(age) => format!("{subject} come from ~/.claude/stats-cache.json because the transcript scan produced none; aggregates were last computed {age}"),
            None => format!("{subject} come from ~/.claude/stats-cache.json because the transcript scan produced none"),
        }
    }
}

pub(super) fn parse_profile(content: &str, now: DateTime<Utc>) -> ClaudeProfile {
    let Ok(root) = serde_json::from_str::<Value>(content) else {
        return ClaudeProfile::failed("local profile: ~/.claude.json could not be parsed");
    };

    let mut profile = ClaudeProfile::default();
    parse_oauth_account(&root, now, &mut profile);
    profile.cached_usage = parse_cached_usage(&root, now, &mut profile.diagnostics);
    profile
}

fn parse_oauth_account(root: &Value, now: DateTime<Utc>, profile: &mut ClaudeProfile) {
    let Some(account) = root.get("oauthAccount").filter(|value| value.is_object()) else {
        profile
            .diagnostics
            .push("plan: the local profile cache has no account section".to_string());
        return;
    };

    profile.plan = plan_name(account, &mut profile.diagnostics);
    profile.subscription_started_at = account_timestamp(account, &mut profile.diagnostics);
    note_profile_age(account, now, &mut profile.diagnostics);
}

/// `organizationType` is used as reported, for example `claude_pro`. It is not
/// rewritten into the `subscription_type` vocabulary of the live source: only
/// one value has been observed and inventing the rest would be a guess.
fn plan_name(account: &Value, diagnostics: &mut Vec<String>) -> Option<String> {
    let value = account.get("organizationType").and_then(Value::as_str)?;
    if !is_name_token(value, PLAN_NAME_MAX_CHARS) {
        diagnostics.push("plan: the local profile value is not a usable plan name".to_string());
        return None;
    }

    diagnostics.push("plan is the organization type from the local profile cache, reported as-is, not from a live response".to_string());
    Some(value.to_string())
}

fn account_timestamp(account: &Value, diagnostics: &mut Vec<String>) -> Option<String> {
    let value = account
        .get("subscriptionCreatedAt")
        .and_then(Value::as_str)?;
    let Some(parsed) = parse_timestamp(value) else {
        diagnostics.push("subscription start date: timestamp could not be parsed".to_string());
        return None;
    };

    Some(format_timestamp(parsed))
}

/// `profileFetchedAt` marks how fresh the two profile fields are. It is not
/// `collected_at` and it never becomes `data_as_of`, which describes the limit
/// snapshot instead.
fn note_profile_age(account: &Value, now: DateTime<Utc>, diagnostics: &mut Vec<String>) {
    let Some(value) = account.get("profileFetchedAt") else {
        return;
    };

    let Some(fetched_at) = epoch_millis(value) else {
        diagnostics.push("local profile cache age: timestamp could not be parsed".to_string());
        return;
    };

    diagnostics.push(format!(
        "local profile cache was last fetched {}",
        format_age(now - fetched_at)
    ));
}

fn parse_cached_usage(
    root: &Value,
    now: DateTime<Utc>,
    diagnostics: &mut Vec<String>,
) -> Option<CachedUsageSnapshot> {
    let Some(cached) = root
        .get("cachedUsageUtilization")
        .filter(|value| value.is_object())
    else {
        diagnostics.push("cached usage snapshot: ~/.claude.json holds no /usage cache".to_string());
        return None;
    };

    let Some(fetched_at) = cached.get("fetchedAtMs").and_then(epoch_millis) else {
        diagnostics.push(
            "cached usage snapshot ignored: it carries no usable fetch timestamp".to_string(),
        );
        return None;
    };

    let Some(utilization) = cached.get("utilization").filter(|value| value.is_object()) else {
        diagnostics.push(
            "cached usage snapshot: the /usage cache holds no utilization payload".to_string(),
        );
        return None;
    };

    diagnostics.push(format!(
        "cached /usage snapshot in ~/.claude.json was fetched {}; it is refreshed only when /usage is opened in the Claude TUI",
        format_age(now - fetched_at)
    ));

    let (credits_total, credits_used) = cached_credits(utilization, diagnostics);
    let (money_used, money_total, money_currency) = cached_money(utilization);

    Some(CachedUsageSnapshot {
        fetched_at,
        windows: cached_windows(utilization, diagnostics),
        credits_total,
        credits_used,
        credits_remaining: difference(credits_total, credits_used),
        money_used,
        money_total,
        money_remaining: difference(money_total, money_used),
        money_currency,
    })
}

fn cached_windows(utilization: &Value, diagnostics: &mut Vec<String>) -> Vec<CachedWindow> {
    let mut windows = Vec::new();

    for (name, window_minutes, kind) in CACHED_WINDOWS {
        let record = utilization.get(name).filter(|value| value.is_object());
        let flat = flat_limit_record(utilization, kind);
        let used_percent = record
            .and_then(|value| percent_field(value, "utilization"))
            .or_else(|| flat.and_then(|value| percent_field(value, "percent")));
        let resets_at = record
            .and_then(|value| timestamp_field(value, "resets_at"))
            .or_else(|| flat.and_then(|value| timestamp_field(value, "resets_at")));

        if used_percent.is_none() && resets_at.is_none() {
            continue;
        }

        if flat
            .and_then(|value| value.get("severity"))
            .and_then(Value::as_str)
            == Some("critical")
        {
            diagnostics.push(format!(
                "cached /usage snapshot marks the {name} window as critical"
            ));
        }

        windows.push(CachedWindow {
            name,
            window_minutes,
            resets_at,
            used_percent,
            used_amount: record.and_then(|value| amount_field(value, "used_dollars")),
            remaining_amount: record.and_then(|value| amount_field(value, "remaining_dollars")),
            total_amount: record.and_then(|value| amount_field(value, "limit_dollars")),
        });
    }

    windows
}

/// `limits[]` overlaps the named windows and is merged into them, never emitted
/// twice. An entry whose `kind` matches no parsed window is not projected.
fn flat_limit_record<'a>(utilization: &'a Value, kind: &str) -> Option<&'a Value> {
    utilization
        .get("limits")?
        .as_array()?
        .iter()
        .find(|record| record.get("kind").and_then(Value::as_str) == Some(kind))
}

/// The credit fields are populated only while the paid overflow allowance is
/// enabled: a monthly limit on a disabled allowance is not a balance.
fn cached_credits(
    utilization: &Value,
    diagnostics: &mut Vec<String>,
) -> (Option<f64>, Option<f64>) {
    let Some(extra) = utilization
        .get("extra_usage")
        .filter(|value| value.is_object())
    else {
        return (None, None);
    };

    if extra.get("is_enabled").and_then(Value::as_bool) != Some(true) {
        diagnostics
            .push("credit fields left empty: the extra usage allowance is disabled".to_string());
        return (None, None);
    }

    let Some(places) = decimal_places(extra, "decimal_places") else {
        diagnostics.push(
            "credit fields left empty: the cached snapshot states no amount scale".to_string(),
        );
        return (None, None);
    };

    (
        minor_amount(extra.get("monthly_limit"), places),
        minor_amount(extra.get("used_credits"), places),
    )
}

fn cached_money(utilization: &Value) -> (Option<f64>, Option<f64>, Option<String>) {
    let Some(spend) = utilization.get("spend").filter(|value| value.is_object()) else {
        return (None, None, None);
    };
    let Some(used) = spend.get("used").filter(|value| value.is_object()) else {
        return (None, None, None);
    };
    let Some(places) = decimal_places(used, "exponent") else {
        return (None, None, None);
    };

    let total = match spend.get("limit") {
        Some(Value::Object(_)) => spend
            .get("limit")
            .and_then(|limit| minor_amount(limit.get("amount_minor"), places)),
        limit => minor_amount(limit, places),
    };

    (
        minor_amount(used.get("amount_minor"), places),
        total,
        currency_code(used.get("currency")),
    )
}

pub(super) fn parse_stats_cache(content: &str, now: DateTime<Utc>) -> ClaudeStatsCache {
    let Ok(root) = serde_json::from_str::<Value>(content) else {
        return ClaudeStatsCache::failed(
            "usage aggregates: ~/.claude/stats-cache.json could not be parsed",
        );
    };

    let mut cache = ClaudeStatsCache {
        sessions_count: root.get("totalSessions").and_then(Value::as_u64),
        turns_count: root.get("totalMessages").and_then(Value::as_u64),
        ..ClaudeStatsCache::default()
    };
    parse_model_usage(&root, &mut cache);
    note_stats_cache_age(&root, now, &mut cache);
    cache
}

fn parse_model_usage(root: &Value, cache: &mut ClaudeStatsCache) {
    let Some(models) = root.get("modelUsage").and_then(Value::as_object) else {
        return;
    };

    let mut input = 0u64;
    let mut output = 0u64;
    let mut cache_read = 0u64;
    let mut cache_creation = 0u64;
    let mut top_model: Option<(String, u64)> = None;
    let mut unusable_model_name = false;

    for (model, record) in models {
        let model_input = token_field(record, "inputTokens");
        let model_output = token_field(record, "outputTokens");
        let model_cache_read = token_field(record, "cacheReadInputTokens");
        let model_cache_creation = token_field(record, "cacheCreationInputTokens");
        let model_total = model_input
            .saturating_add(model_output)
            .saturating_add(model_cache_read)
            .saturating_add(model_cache_creation);

        input = input.saturating_add(model_input);
        output = output.saturating_add(model_output);
        cache_read = cache_read.saturating_add(model_cache_read);
        cache_creation = cache_creation.saturating_add(model_cache_creation);

        if !is_name_token(model, MODEL_NAME_MAX_CHARS) {
            unusable_model_name = true;
            continue;
        }

        if top_model
            .as_ref()
            .is_none_or(|(_, best)| model_total > *best)
        {
            top_model = Some((model.clone(), model_total));
        }
    }

    if models.is_empty() {
        return;
    }

    if unusable_model_name {
        cache.diagnostics.push(
            "top model: a name in the usage aggregates is not a usable model name".to_string(),
        );
    }

    cache.input_tokens = Some(input);
    cache.output_tokens = Some(output);
    cache.cache_read_tokens = Some(cache_read);
    cache.cache_creation_tokens = Some(cache_creation);
    cache.total_tokens = Some(
        input
            .saturating_add(output)
            .saturating_add(cache_read)
            .saturating_add(cache_creation),
    );
    cache.top_model = top_model.map(|(model, _)| model);
}

/// `lastComputedDate` is a plain `YYYY-MM-DD` day. It is reported as an age in
/// `diagnostics` and never becomes `data_as_of`.
fn note_stats_cache_age(root: &Value, now: DateTime<Utc>, cache: &mut ClaudeStatsCache) {
    let Some(value) = root.get("lastComputedDate") else {
        return;
    };

    let computed = value
        .as_str()
        .and_then(|day| chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").ok())
        .and_then(|day| day.and_hms_opt(0, 0, 0))
        .map(|day| day.and_utc());
    let Some(computed) = computed else {
        cache
            .diagnostics
            .push("usage aggregates age: the computed date could not be parsed".to_string());
        return;
    };

    cache.computed_age = Some(format_age(now - computed));
}

fn token_field(record: &Value, key: &str) -> u64 {
    record.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn is_name_token(value: &str, max_chars: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= max_chars
        && value
            .chars()
            .all(|item| item.is_ascii_alphanumeric() || matches!(item, '-' | '_' | '.'))
}

fn percent_field(record: &Value, key: &str) -> Option<f64> {
    let value = record.get(key)?.as_f64()?;
    (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value)
}

fn amount_field(record: &Value, key: &str) -> Option<f64> {
    let value = record.get(key)?.as_f64()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

fn timestamp_field(record: &Value, key: &str) -> Option<DateTime<Utc>> {
    parse_timestamp(record.get(key)?.as_str()?)
}

fn decimal_places(record: &Value, key: &str) -> Option<i64> {
    let places = record.get(key)?.as_i64()?;
    (0..=MAX_DECIMAL_PLACES).contains(&places).then_some(places)
}

fn minor_amount(value: Option<&Value>, places: i64) -> Option<f64> {
    let minor = value?.as_f64()?;
    if !minor.is_finite() {
        return None;
    }

    Some(minor / 10f64.powi(places as i32))
}

fn currency_code(value: Option<&Value>) -> Option<String> {
    let code = value?.as_str()?;
    let usable = code.len() == 3 && code.chars().all(|item| item.is_ascii_uppercase());
    usable.then(|| code.to_string())
}

fn difference(total: Option<f64>, used: Option<f64>) -> Option<f64> {
    Some((total? - used?).max(0.0))
}

fn epoch_millis(value: &Value) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_millis(value.as_i64()?)
}

pub(super) fn format_age(age: Duration) -> String {
    if age < Duration::zero() {
        return "in the future".to_string();
    }

    match (age.num_days(), age.num_hours(), age.num_minutes()) {
        (days, _, _) if days > 0 => format!("{days}d ago"),
        (_, hours, _) if hours > 0 => format!("{hours}h ago"),
        (_, _, minutes) if minutes > 0 => format!("{minutes}m ago"),
        _ => "less than a minute ago".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn extracts_server_reset_anchor_from_rate_limits_payload() {
        let record: Value = serde_json::from_str(
            r#"{"type":"assistant","payload":{"rate_limits":{"five_hour":{"resets_at":"1782721800"}}}}"#,
        )
        .expect("parse record");

        let anchor = extract_server_reset_anchor(&record).expect("server reset anchor");

        assert_eq!(
            anchor
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-29T08:30:00Z"
        );
        assert_eq!(
            anchor.source_path,
            "/payload/rate_limits/five_hour/resets_at"
        );
    }

    #[test]
    fn extracts_server_reset_anchor_from_nested_429_usage_limit_payload() {
        let record: Value = serde_json::from_str(
            r#"{"type":"error","payload":{"status":429,"error":{"usage_limit":{"reset_time":"2026-06-29T08:30:00Z"}}}}"#,
        )
        .expect("parse record");

        let anchor = extract_server_reset_anchor(&record).expect("server reset anchor");

        assert_eq!(
            anchor
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-29T08:30:00Z"
        );
        assert_eq!(anchor.source_path, "/payload/error/usage_limit/reset_time");
    }
}
