use crate::types::{CliAuthorization, LimitInfo, StructuredSourceInfo};

use super::super::time::{format_user_timestamp, TimeContext};

pub struct ProviderBlock {
    pub provider_label: String,
    pub body: String,
}

pub fn provider_label(info: &StructuredSourceInfo) -> String {
    info.provider.to_ascii_uppercase()
}

pub fn format_data_as_of(info: &StructuredSourceInfo) -> String {
    let source = source_label_for_display(&info.source);
    match info.data_as_of.as_deref() {
        Some(value) => {
            let context = TimeContext::from_structured(info);
            format!(
                "Source {source}: {}",
                format_user_timestamp(value, &context)
            )
        }
        None => format!("Source {source}: unknown"),
    }
}

pub fn format_unavailable_block(info: &StructuredSourceInfo) -> String {
    if let Some(auth) = info.status.cli_authorization {
        return format_cli_authorization_block(auth, info);
    }

    let message = info.status.message.as_deref().unwrap_or("unavailable");
    format!("Unavailable: {message}\n{}", format_data_as_of(info))
}

fn format_cli_authorization_block(auth: CliAuthorization, info: &StructuredSourceInfo) -> String {
    let headline = match auth {
        CliAuthorization::Codex => "You\u{2019}re not signed in to Codex CLI.",
        CliAuthorization::Claude => "You\u{2019}re not signed in to Claude CLI.",
    };

    format!(
        "{headline}\nRun it: `{}`\n{}",
        auth.login_command(),
        format_data_as_of(info)
    )
}

pub fn source_label_for_display(source: &str) -> String {
    source.replace('_', "-")
}

/// Cursor `plan_usage` and `included_spend` stay in structured data but are not
/// shown in user-facing surfaces (terminal, desktop, notifications).
pub fn is_limit_shown_to_user(limit: &LimitInfo) -> bool {
    !matches!(
        limit.name.trim().to_ascii_lowercase().as_str(),
        "plan_usage" | "included_spend" | "plan" | "total"
    )
}

/// Compact window label for the terminal (`LIMIT_WINDOW_WIDTH` is 4).
pub fn window_label_for_display(limit: &LimitInfo) -> String {
    if let Some(minutes) = limit.window_minutes {
        return compact_window_from_minutes(minutes);
    }

    if let Some(label) = limit.window_label.as_deref() {
        let compact = compact_window_label(label);
        if compact.chars().count() <= 4 {
            return compact;
        }
    }

    compact_name_label(&limit.name)
}

/// Desktop card labels — full Cursor pool names, compact otherwise.
pub fn window_label_for_desktop(limit: &LimitInfo) -> String {
    match limit.name.trim().to_ascii_lowercase().as_str() {
        "auto" | "api_models" | "api" | "api models" => limit_type_label(&limit.name),
        _ => window_label_for_display(limit),
    }
}

/// Notification `$TYPE` label — full Cursor pool names.
pub fn limit_type_label(limit_name: &str) -> String {
    match limit_name.trim().to_ascii_lowercase().as_str() {
        "5h" | "five_hour" | "five hour" | "session" | "primary" => "5h".to_string(),
        "weekly" | "week" | "7d" | "seven_day" | "seven day" | "secondary" => "weekly".to_string(),
        "auto" => "Cursor Models".to_string(),
        "api" | "api_models" | "api models" => "Other Models".to_string(),
        "" => "limit".to_string(),
        value => value.replace('_', " "),
    }
}

fn compact_window_from_minutes(minutes: u64) -> String {
    match minutes {
        300 => "5h".to_string(),
        10080 => "7d".to_string(),
        _ => format!("{minutes}m"),
    }
}

fn compact_window_label(label: &str) -> String {
    let normalized = label.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "5h" | "5-hour window" | "5 hour" | "five_hour" | "primary window" | "current session" => {
            "5h".to_string()
        }
        "7d" | "7-day window" | "7 day" | "seven_day" | "weekly" | "secondary window"
        | "current week" => "7d".to_string(),
        _ if normalized.contains("5") && normalized.contains("hour") => "5h".to_string(),
        _ if normalized.contains("7") && normalized.contains("day") => "7d".to_string(),
        _ if normalized.contains("week") => "7d".to_string(),
        _ => label.to_string(),
    }
}

fn compact_name_label(name: &str) -> String {
    match name {
        "5h limit" => "5h".to_string(),
        "Weekly limit" => "7d".to_string(),
        "auto" => "Curs".to_string(),
        "api_models" => "Oth".to_string(),
        other => {
            let trimmed = other.trim();
            if trimmed.chars().count() <= 4 {
                trimmed.to_string()
            } else {
                trimmed.chars().take(4).collect()
            }
        }
    }
}
