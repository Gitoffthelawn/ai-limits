use chrono::{DateTime, Duration, Utc};

use super::parse::parse_timestamp;

pub(super) const CLAUDE_LOCAL_MAX5_TOKEN_LIMIT: u64 = 88_000;
pub(super) const CLAUDE_LOCAL_SESSION_WINDOW_MINUTES: u64 = 5 * 60;

#[derive(Default)]
pub(super) struct ClaudeLocalUsage {
    pub(super) files: usize,
    pub(super) sessions: std::collections::HashSet<String>,
    pub(super) turns: usize,
    pub(super) input_tokens: u64,
    pub(super) output_tokens: u64,
    pub(super) cache_read_tokens: u64,
    pub(super) cache_creation_tokens: u64,
    pub(super) latest_timestamp: Option<String>,
    pub(super) models: std::collections::HashMap<String, u64>,
    pub(super) turns_by_time: Vec<TurnUsage>,
    pub(super) latest_server_reset_anchor: Option<ServerResetAnchor>,
}

#[derive(Clone)]
pub(super) struct TurnUsage {
    pub(super) session_id: String,
    pub(super) timestamp: Option<String>,
    pub(super) model: Option<String>,
    pub(super) message_id: Option<String>,
    pub(super) input_tokens: u64,
    pub(super) output_tokens: u64,
    pub(super) cache_read_tokens: u64,
    pub(super) cache_creation_tokens: u64,
}

pub(super) struct ActiveSessionLimit {
    pub(super) resets_at: DateTime<Utc>,
    pub(super) used_tokens: u64,
    pub(super) token_limit: u64,
    pub(super) reset_source: ResetSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ServerResetAnchor {
    pub(super) resets_at: DateTime<Utc>,
    pub(super) source_path: String,
}

impl PartialOrd for ServerResetAnchor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ServerResetAnchor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.resets_at.cmp(&other.resets_at)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResetSource {
    ServerAnchor,
    TranscriptEstimate,
}

pub(super) fn active_session_limit(
    usage: &ClaudeLocalUsage,
    now: DateTime<Utc>,
) -> Option<ActiveSessionLimit> {
    let mut turns = usage
        .turns_by_time
        .iter()
        .filter_map(|turn| {
            let timestamp = turn.timestamp.as_deref().and_then(parse_timestamp)?;
            Some((timestamp, turn))
        })
        .collect::<Vec<_>>();
    turns.sort_by_key(|(timestamp, _)| *timestamp);

    let mut current: Option<ActiveSessionLimit> = None;
    let mut previous_timestamp: Option<DateTime<Utc>> = None;
    let session_duration = Duration::minutes(CLAUDE_LOCAL_SESSION_WINDOW_MINUTES as i64);

    if let Some(anchor) = usage
        .latest_server_reset_anchor
        .as_ref()
        .filter(|anchor| anchor.resets_at > now)
    {
        let resets_at = anchor.resets_at;
        let window_start = resets_at - session_duration;
        let used_tokens = turns
            .iter()
            .filter(|(timestamp, _)| *timestamp >= window_start && *timestamp < resets_at)
            .map(|(_, turn)| turn.input_tokens + turn.output_tokens)
            .sum();

        return Some(ActiveSessionLimit {
            resets_at,
            used_tokens,
            token_limit: CLAUDE_LOCAL_MAX5_TOKEN_LIMIT,
            reset_source: ResetSource::ServerAnchor,
        });
    }

    for (timestamp, turn) in turns {
        let should_start_new = current
            .as_ref()
            .is_none_or(|block| timestamp >= block.resets_at)
            || previous_timestamp.is_some_and(|previous| timestamp - previous >= session_duration);

        if should_start_new {
            current = Some(ActiveSessionLimit {
                resets_at: timestamp + session_duration,
                used_tokens: 0,
                token_limit: CLAUDE_LOCAL_MAX5_TOKEN_LIMIT,
                reset_source: ResetSource::TranscriptEstimate,
            });
        }

        if let Some(block) = current.as_mut() {
            block.used_tokens += turn.input_tokens + turn.output_tokens;
        }
        previous_timestamp = Some(timestamp);
    }

    current.filter(|block| block.resets_at > now)
}

#[cfg(test)]
pub(super) fn sample_usage() -> ClaudeLocalUsage {
    let mut usage = ClaudeLocalUsage {
        files: 2,
        ..ClaudeLocalUsage::default()
    };
    usage.sessions.extend(["s1".to_string(), "s2".to_string()]);
    usage.turns = 5;
    usage.input_tokens = 100;
    usage.output_tokens = 40;
    usage.cache_read_tokens = 10;
    usage.cache_creation_tokens = 5;
    usage.latest_timestamp = Some("2026-06-28T10:01:00Z".to_string());
    usage.models.insert("claude-sonnet-4-6".to_string(), 3);
    usage.models.insert("claude-haiku-4-5".to_string(), 2);
    usage
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::claude_local::project::limit_info_from_active_session;

    #[test]
    fn transcript_estimate_does_not_round_reset_down_to_hour() {
        let mut usage = ClaudeLocalUsage::default();
        usage.turns_by_time.push(TurnUsage {
            session_id: "s1".to_string(),
            timestamp: Some("2026-06-28T10:37:12Z".to_string()),
            model: None,
            message_id: Some("m1".to_string()),
            input_tokens: 100,
            output_tokens: 40,
            cache_read_tokens: 1_000,
            cache_creation_tokens: 2_000,
        });

        let now = parse_timestamp("2026-06-28T11:00:00Z").expect("parse now");
        let limit = active_session_limit(&usage, now).expect("active limit");

        assert_eq!(
            limit
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-28T15:37:12Z"
        );
        assert_eq!(limit.used_tokens, 140);
        assert_eq!(limit.reset_source, ResetSource::TranscriptEstimate);

        let info = limit_info_from_active_session(&limit);
        assert_eq!(info.name.as_str(), "5h local estimate (estimated reset)");
    }

    #[test]
    fn server_reset_anchor_overrides_transcript_estimated_window() {
        let mut usage = ClaudeLocalUsage {
            latest_server_reset_anchor: Some(ServerResetAnchor {
                resets_at: parse_timestamp("2026-06-28T15:00:00Z").expect("parse anchor"),
                source_path: "/payload/rate_limits/five_hour/resets_at".to_string(),
            }),
            ..ClaudeLocalUsage::default()
        };
        usage.turns_by_time.push(TurnUsage {
            session_id: "old".to_string(),
            timestamp: Some("2026-06-28T09:59:59Z".to_string()),
            model: None,
            message_id: Some("old".to_string()),
            input_tokens: 1_000,
            output_tokens: 1_000,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        });
        usage.turns_by_time.push(TurnUsage {
            session_id: "current".to_string(),
            timestamp: Some("2026-06-28T10:00:00Z".to_string()),
            model: None,
            message_id: Some("current".to_string()),
            input_tokens: 20,
            output_tokens: 5,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        });

        let now = parse_timestamp("2026-06-28T11:00:00Z").expect("parse now");
        let limit = active_session_limit(&usage, now).expect("active limit");

        assert_eq!(
            limit
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-28T15:00:00Z"
        );
        assert_eq!(limit.used_tokens, 25);
        assert_eq!(limit.reset_source, ResetSource::ServerAnchor);

        let info = limit_info_from_active_session(&limit);
        assert_eq!(
            info.name.as_str(),
            "5h local estimate (server reset anchor)"
        );
    }
}
