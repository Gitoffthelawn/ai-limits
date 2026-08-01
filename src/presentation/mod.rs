mod common;
mod limits;
mod time;
mod usage;

pub use common::{
    normalize_percent, remaining_percent_for_display, source_label_for_display,
    window_label_for_display, ColorConfig, ProviderBlock,
};
pub use limits::limits_block;
pub use time::{format_user_timestamp, TimeContext};
pub use usage::usage_block;

pub fn format_raw_output(data: &crate::types::SourceData) -> String {
    match data.raw.as_deref() {
        Some(raw) if !raw.trim().is_empty() => raw.to_string(),
        _ if !data.structured.raw_data_available => data
            .structured
            .status
            .message
            .clone()
            .unwrap_or_else(|| "raw data unavailable".to_string()),
        _ => data
            .structured
            .status
            .message
            .clone()
            .unwrap_or_else(|| "raw data unavailable".to_string()),
    }
}

pub fn format_structured_output(data: &crate::types::SourceData) -> String {
    serde_json::to_string_pretty(&data.structured)
        .unwrap_or_else(|error| format!("failed to serialize structured data: {error}"))
}

#[cfg(test)]
mod tests;
