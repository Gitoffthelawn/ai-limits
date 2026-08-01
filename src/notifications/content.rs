use serde::{Deserialize, Serialize};

use crate::presentation::{format_user_timestamp, TimeContext};

use super::kinds::{LimitNotificationKind, NotificationColor};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Notification {
    pub dedupe_key: String,
    pub title: String,
    pub subtitle: String,
    pub message: String,
    pub color: NotificationColor,
}

impl Notification {
    pub fn limit(
        provider: &str,
        source: &str,
        limit_name: &str,
        kind: LimitNotificationKind,
        remaining_percent: f64,
        resets_at: Option<&str>,
        time_context: &TimeContext,
    ) -> Self {
        let provider = provider_label(provider);
        let type_label = limit_type_label(limit_name);
        let remaining_percent = display_percent(remaining_percent);
        Self {
            dedupe_key: format!(
                "{}|{}|{}|{}",
                provider,
                source,
                type_label,
                kind.remaining_percent()
            ),
            title: format!("{} AI Limits", kind.emoji()),
            subtitle: format!("{provider} {type_label} - {remaining_percent}% left"),
            message: format!("reset {}", reset_label(resets_at, time_context)),
            color: kind.color(),
        }
    }

    pub fn test(kind: LimitNotificationKind) -> Self {
        let remaining_percent = kind.remaining_percent();
        Self {
            dedupe_key: format!("test|{remaining_percent}"),
            title: format!("{} AI Limits", kind.emoji()),
            subtitle: format!("AI Limits test - {remaining_percent}% left"),
            message: "reset unknown".to_string(),
            color: kind.color(),
        }
    }
}

fn provider_label(provider: &str) -> String {
    match provider.trim().to_ascii_lowercase().as_str() {
        "codex" => "Codex".to_string(),
        "claude" => "Claude".to_string(),
        "cursor" => "Cursor".to_string(),
        "" => "AI Limits".to_string(),
        _ => title_case(provider),
    }
}

fn limit_type_label(limit_name: &str) -> String {
    match limit_name.trim().to_ascii_lowercase().as_str() {
        "5h" | "five_hour" | "five hour" | "session" | "primary" => "5h".to_string(),
        "weekly" | "week" | "7d" | "seven_day" | "seven day" | "secondary" => "weekly".to_string(),
        "auto" => "auto".to_string(),
        "plan" | "total" => "plan".to_string(),
        "api" | "api_models" | "api models" => "api".to_string(),
        "" => "limit".to_string(),
        value => value.replace('_', " "),
    }
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    first.to_uppercase().collect::<String>() + chars.as_str()
}

fn display_percent(remaining_percent: f64) -> u8 {
    remaining_percent.clamp(0.0, 100.0).round() as u8
}

fn reset_label(value: Option<&str>, time_context: &TimeContext) -> String {
    value
        .map(|value| format_user_timestamp(value, time_context))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
