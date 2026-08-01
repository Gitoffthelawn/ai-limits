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
        "auto" => "auto".to_string(),
        "api_models" => "api".to_string(),
        "plan_usage" => "plan".to_string(),
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
