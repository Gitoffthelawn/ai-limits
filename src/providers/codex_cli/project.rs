use chrono::Utc;

use crate::types::{
    AccountInfo, CliAuthorization, LimitInfo, SourceData, SourceStatus, StructuredSourceInfo,
    UsageInfo,
};

use super::parse::{
    normalize_line, output_requires_authorization, parse_available_reset_count, parse_credits_line,
    parse_limit_line,
};

const PROVIDER: &str = "codex";
const SOURCE: &str = "codex_cli";
const SOURCE_LINK: &str = "https://developers.openai.com/codex/cli";
pub(super) const SETUP_LINK: &str = SOURCE_LINK;

pub fn build_structured(raw: &str) -> StructuredSourceInfo {
    let mut limits = Vec::new();
    let mut account = AccountInfo::default();
    let mut available_limit_resets = None;
    let mut diagnostics = Vec::new();
    let mut found_data = false;

    for raw_line in raw.lines() {
        let normalized = normalize_line(raw_line);

        if normalized.starts_with("5h limit:") {
            if let Some(limit) = parse_limit_line("5h limit", "5h", 300, &normalized) {
                upsert_limit(&mut limits, limit);
                found_data = true;
            } else {
                diagnostics.push("could not parse 5h limit line".to_string());
            }
        } else if normalized.starts_with("Weekly limit:") {
            if let Some(limit) = parse_limit_line("Weekly limit", "weekly", 10080, &normalized) {
                upsert_limit(&mut limits, limit);
                found_data = true;
            } else {
                diagnostics.push("could not parse weekly limit line".to_string());
            }
        } else if normalized.starts_with("Credits:") {
            match parse_credits_line(&normalized) {
                Some(credits) => {
                    account.credits_remaining = Some(credits);
                    found_data = true;
                }
                None => diagnostics.push("could not parse credits line".to_string()),
            }
        } else if let Some(count) = parse_available_reset_count(&normalized) {
            available_limit_resets = Some(count);
            found_data = true;
        }
    }

    let auth_required = output_requires_authorization(raw);
    let (access_available, data_available, message, cli_authorization) = if found_data {
        (true, true, None, None)
    } else if auth_required {
        (
            false,
            false,
            Some(format!(
                "Codex CLI is installed but not authorized; run `codex login` and try again. Setup: {SETUP_LINK}"
            )),
            Some(CliAuthorization::Codex),
        )
    } else if raw.trim().is_empty() {
        (
            true,
            false,
            Some("Codex CLI returned empty output".to_string()),
            None,
        )
    } else {
        (
            true,
            false,
            Some("supported limit lines not found in Codex CLI output".to_string()),
            None,
        )
    };
    let collected_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
    let data_as_of = data_available.then(|| collected_at.clone()).flatten();

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available,
            access_available,
            message,
            cli_authorization,
        },
        raw_data_available: !raw.trim().is_empty(),
        collected_at,
        data_as_of,
        account,
        limits,
        available_limit_resets,
        usage: UsageInfo::default(),
        diagnostics,
    }
}

pub(super) fn authorization_required_source_data() -> SourceData {
    unavailable_source_data(
        None,
        &format!(
            "Codex CLI is installed but not authorized; run `codex login` and try again. Setup: {SETUP_LINK}"
        ),
        Some(CliAuthorization::Codex),
    )
}

pub(super) fn unavailable_source_data(
    raw: Option<String>,
    message: &str,
    cli_authorization: Option<CliAuthorization>,
) -> SourceData {
    let raw_data_available = raw.as_ref().is_some_and(|value| !value.trim().is_empty());

    SourceData {
        raw,
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some(message.to_string()),
                cli_authorization,
            },
            raw_data_available,
            collected_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        },
        stderr: String::new(),
    }
}

fn upsert_limit(limits: &mut Vec<LimitInfo>, limit: LimitInfo) {
    if let Some(index) = limits
        .iter()
        .position(|existing| existing.name == limit.name)
    {
        limits[index] = limit;
    } else {
        limits.push(limit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OUTPUT: &str = "\
5h limit: [░░░░░░░░░░░░░░░░░░░░] 0% left (resets 07:59)
Weekly limit: [█████████████████░░░] 84% left (resets 02:59 on 6 Jul)
Credits: 335 credits
";

    #[test]
    fn build_structured_parses_representative_cli_output() {
        let info = build_structured(SAMPLE_OUTPUT);

        assert_eq!(info.provider, "codex");
        assert_eq!(info.source, "codex_cli");
        assert!(info.status.access_available);
        assert!(info.status.data_available);
        assert!(info.raw_data_available);
        assert!(info.collected_at.is_some());
        assert_eq!(info.data_as_of.as_deref(), info.collected_at.as_deref());
        assert_eq!(info.limits.len(), 2);
        assert_eq!(info.account.credits_remaining, Some(335.0));
        assert_eq!(info.available_limit_resets, None);

        let five_hour = &info.limits[0];
        assert_eq!(five_hour.name, "5h limit");
        assert_eq!(five_hour.window_label.as_deref(), Some("5h"));
        assert_eq!(five_hour.window_minutes, Some(300));
        assert_eq!(five_hour.remaining_percent, Some(0.0));
        assert_eq!(five_hour.used_percent, Some(100.0));
        assert_eq!(five_hour.resets_at.as_deref(), Some("07:59"));

        let weekly = &info.limits[1];
        assert_eq!(weekly.name, "Weekly limit");
        assert_eq!(weekly.window_label.as_deref(), Some("weekly"));
        assert_eq!(weekly.window_minutes, Some(10080));
        assert_eq!(weekly.remaining_percent, Some(84.0));
        assert_eq!(weekly.used_percent, Some(16.0));
        assert_eq!(weekly.resets_at.as_deref(), Some("02:59 on 6 Jul"));
    }

    #[test]
    fn build_structured_reports_missing_data_when_output_has_no_limits() {
        let info = build_structured("OpenAI Codex\n> welcome\n");

        assert!(info.status.access_available);
        assert!(!info.status.data_available);
        assert!(info.raw_data_available);
        assert_eq!(
            info.status.message.as_deref(),
            Some("supported limit lines not found in Codex CLI output")
        );
        assert!(info.limits.is_empty());
        assert!(info.account.credits_remaining.is_none());
        assert!(info.collected_at.is_some());
        assert!(info.data_as_of.is_none());
    }

    #[test]
    fn build_structured_reports_empty_output() {
        let info = build_structured("");

        assert!(info.status.access_available);
        assert!(!info.status.data_available);
        assert!(!info.raw_data_available);
        assert_eq!(
            info.status.message.as_deref(),
            Some("Codex CLI returned empty output")
        );
    }

    #[test]
    fn build_structured_marks_authorization_required() {
        let info = build_structured("OpenAI Codex\nSign in with ChatGPT to continue\n");

        assert!(!info.status.access_available);
        assert!(!info.status.data_available);
        assert!(info.raw_data_available);
        assert_eq!(info.status.cli_authorization, Some(CliAuthorization::Codex));
        assert_eq!(
            info.status.message.as_deref(),
            Some("Codex CLI is installed but not authorized; run `codex login` and try again. Setup: https://developers.openai.com/codex/cli")
        );
    }

    #[test]
    fn authorization_required_source_data_does_not_launch_the_interactive_cli() {
        let data = authorization_required_source_data();

        assert_eq!(
            data.structured.status.cli_authorization,
            Some(CliAuthorization::Codex)
        );
        assert!(!data.structured.status.access_available);
        assert!(!data.structured.status.data_available);
    }

    #[test]
    fn unavailable_source_data_marks_cli_not_installed() {
        let data = unavailable_source_data(
            None,
            "Codex CLI is not installed or is not available in PATH; install `codex` and try again. Setup: https://developers.openai.com/codex/cli",
            None,
        );

        assert!(!data.structured.status.access_available);
        assert!(!data.structured.status.data_available);
        assert!(!data.structured.raw_data_available);
        assert_eq!(
            data.structured.status.message.as_deref(),
            Some("Codex CLI is not installed or is not available in PATH; install `codex` and try again. Setup: https://developers.openai.com/codex/cli")
        );
    }

    #[test]
    fn build_structured_deduplicates_repeated_limit_lines() {
        let output = "\
5h limit: [████░░░░░░░░░░░░░░░░] 35% left (resets 03:48)
Weekly limit: [████████████░░░░░░░░] 59% left (resets 02:59 on 6 Jul)
5h limit: [████░░░░░░░░░░░░░░░░] 35% left (resets 03:48)
Weekly limit: [████████████░░░░░░░░] 59% left (resets 02:59 on 6 Jul)
Credits: 301 credits
";
        let info = build_structured(output);

        assert_eq!(info.limits.len(), 2);
        assert_eq!(info.limits[0].name, "5h limit");
        assert_eq!(info.limits[1].name, "Weekly limit");
        assert_eq!(info.account.credits_remaining, Some(301.0));
    }

    #[test]
    fn build_structured_keeps_latest_duplicate_limit_values() {
        let output = "\
5h limit: 10% left (resets 07:59)
5h limit: 35% left (resets 03:48)
";
        let info = build_structured(output);

        assert_eq!(info.limits.len(), 1);
        assert_eq!(info.limits[0].remaining_percent, Some(35.0));
    }

    #[test]
    fn build_structured_adds_diagnostics_for_unparseable_limit_line() {
        let info = build_structured("5h limit: unavailable\nCredits: 10 credits\n");

        assert!(info.status.data_available);
        assert_eq!(info.limits.len(), 0);
        assert_eq!(info.account.credits_remaining, Some(10.0));
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("5h limit")));
    }

    #[test]
    fn build_structured_parses_available_manual_resets_from_usage_view() {
        let info = build_structured(
            "5h limit: 35% left (resets 03:48)\nYou have 2 usage limit resets available\n",
        );

        assert_eq!(info.available_limit_resets, Some(2));
    }

    #[test]
    fn build_structured_parses_zero_available_manual_resets() {
        let info = build_structured("You have 0 usage limit resets available\n");

        assert_eq!(info.available_limit_resets, Some(0));
    }

    #[test]
    fn build_structured_ignores_unparseable_manual_reset_count() {
        let info = build_structured("You have several usage limit resets available\n");

        assert_eq!(info.available_limit_resets, None);
    }
}
