use ai_limits::get_limits::{SourcePriority, UiSourcePlanOptions};
use ai_limits::presentation::{
    format_user_timestamp, normalize_percent, remaining_percent_for_display,
    source_label_for_display, window_label_for_display, TimeContext,
};
use ai_limits::types::StructuredSourceInfo;

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimitsQuery {
    pub enabled_codex: bool,
    pub enabled_claude: bool,
    pub enabled_cursor: bool,
    pub source_priority: SourcePriority,
    pub notifications_enabled: bool,
}

impl Default for ProviderLimitsQuery {
    fn default() -> Self {
        let defaults = UiSourcePlanOptions::default();
        Self {
            enabled_codex: defaults.enabled_codex,
            enabled_claude: defaults.enabled_claude,
            enabled_cursor: defaults.enabled_cursor,
            source_priority: defaults.source_priority,
            notifications_enabled: true,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimits {
    id: String,
    label: String,
    source_id: Option<String>,
    data_timestamp: Option<String>,
    selected_update_frequency: String,
    limits: Vec<ProviderLimitRow>,
    credits_remaining: Option<f64>,
    available_limit_resets: Option<u64>,
    error_message: Option<String>,
    no_fresh_data: bool,
    authorization_required: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimitRow {
    label: String,
    remaining_percentage: f64,
    reset_time: Option<String>,
}

pub(super) fn provider_limits_from_structured(
    id: &str,
    info: &StructuredSourceInfo,
) -> ProviderLimits {
    let time_context = TimeContext::from_structured(info);
    let limits: Vec<ProviderLimitRow> = info
        .limits
        .iter()
        .filter_map(|limit| {
            let remaining = normalize_percent(remaining_percent_for_display(limit)?);
            Some(ProviderLimitRow {
                label: window_label_for_display(limit),
                remaining_percentage: remaining,
                reset_time: limit
                    .resets_at
                    .as_deref()
                    .map(|value| format_user_timestamp(value, &time_context)),
            })
        })
        .collect();

    let no_fresh_data =
        info.status.access_available && limits.is_empty() && info.available_limit_resets.is_none();

    let authorization_required = info
        .status
        .cli_authorization
        .map(|auth| auth.provider_id().to_string());

    let error_message = if authorization_required.is_some()
        || (info.status.access_available && info.status.data_available)
    {
        None
    } else {
        info.status
            .message
            .clone()
            .or_else(|| Some("No usable limit data".to_string()))
    };

    ProviderLimits {
        id: id.to_string(),
        label: provider_label(id),
        source_id: Some(source_label_for_display(&info.source)),
        data_timestamp: Some(
            info.data_as_of
                .as_deref()
                .map(|value| format_user_timestamp(value, &time_context))
                .unwrap_or_else(|| "unknown".to_string()),
        ),
        selected_update_frequency: "5 min".to_string(),
        limits,
        credits_remaining: info.account.credits_remaining,
        available_limit_resets: info.available_limit_resets,
        error_message,
        no_fresh_data,
        authorization_required,
    }
}

pub(super) fn provider_error(id: &str, message: String) -> ProviderLimits {
    ProviderLimits {
        id: id.to_string(),
        label: provider_label(id),
        source_id: None,
        data_timestamp: None,
        selected_update_frequency: "5 min".to_string(),
        limits: Vec::new(),
        credits_remaining: None,
        available_limit_resets: None,
        error_message: Some(message),
        no_fresh_data: false,
        authorization_required: None,
    }
}

fn provider_label(id: &str) -> String {
    let mut characters = id.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_limits::types::{AccountInfo, CliAuthorization, SourceStatus, UsageInfo};

    fn structured_with_resets(available_limit_resets: Option<u64>) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_cli".to_string(),
            source_link: String::new(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
                cli_authorization: None,
            },
            raw_data_available: true,
            collected_at: Some("2026-07-01T12:00:00Z".to_string()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    fn structured_cli_authorization(auth: CliAuthorization) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: auth.provider_id().to_string(),
            source: format!("{}_cli", auth.provider_id()),
            source_link: String::new(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some("technical auth message".to_string()),
                cli_authorization: Some(auth),
            },
            raw_data_available: true,
            collected_at: Some("2026-07-01T12:00:00Z".to_string()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn projects_null_available_limit_resets_and_marks_no_fresh_data() {
        let response = provider_limits_from_structured("codex", &structured_with_resets(None));

        assert!(response.available_limit_resets.is_none());
        assert!(response.no_fresh_data);
        assert!(response.authorization_required.is_none());
    }

    #[test]
    fn projects_zero_available_limit_resets_as_known_limit_data() {
        let response = provider_limits_from_structured("codex", &structured_with_resets(Some(0)));

        assert_eq!(response.available_limit_resets, Some(0));
        assert!(!response.no_fresh_data);
    }

    #[test]
    fn projects_known_positive_available_limit_resets() {
        let response = provider_limits_from_structured("codex", &structured_with_resets(Some(1)));

        assert_eq!(response.available_limit_resets, Some(1));
        assert!(!response.no_fresh_data);
    }

    #[test]
    fn projects_codex_cli_authorization_without_error_message() {
        let response = provider_limits_from_structured(
            "codex",
            &structured_cli_authorization(CliAuthorization::Codex),
        );

        assert_eq!(response.authorization_required.as_deref(), Some("codex"));
        assert!(response.error_message.is_none());
        assert!(!response.no_fresh_data);
        assert!(response.limits.is_empty());
    }

    #[test]
    fn projects_claude_cli_authorization_without_error_message() {
        let response = provider_limits_from_structured(
            "claude",
            &structured_cli_authorization(CliAuthorization::Claude),
        );

        assert_eq!(response.authorization_required.as_deref(), Some("claude"));
        assert!(response.error_message.is_none());
        assert!(!response.no_fresh_data);
    }
}
