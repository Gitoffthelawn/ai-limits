use chrono::Utc;

use crate::types::{
    CliAuthorization, ProviderRun, SourceData, SourceStatus, StructuredSourceInfo, UsageInfo,
};

use super::parse::parse_claude_cli_output;
use super::{PROVIDER, SETUP_LINK, SOURCE, SOURCE_LINK};

pub fn build_source_data(run: &ProviderRun) -> SourceData {
    let mut structured = structured_from_output(&run.compacted_stdout);
    if !run.stderr.trim().is_empty() {
        structured
            .diagnostics
            .push(format!("stderr: {}", run.stderr.trim()));
    }

    SourceData {
        raw: Some(run.compacted_stdout.clone()),
        structured,
        stderr: run.stderr.clone(),
    }
}

pub fn structured_from_output(stdout: &str) -> StructuredSourceInfo {
    let collected_at = utc_now();
    let parsed = parse_claude_cli_output(stdout);
    let raw_data_available = !stdout.is_empty();

    let (status, limits, usage, diagnostics) = if parsed.setup_required {
        (
            SourceStatus {
                data_available: false,
                access_available: false,
                message: Some(format!(
                    "Claude CLI is installed but not authorized; run `claude login` and try again. Setup: {SETUP_LINK}"
                )),
                cli_authorization: Some(CliAuthorization::Claude),
            },
            Vec::new(),
            UsageInfo::default(),
            Vec::new(),
        )
    } else if parsed.has_usage_data() {
        (
            SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
                cli_authorization: None,
            },
            parsed.limits,
            parsed.usage,
            parsed.diagnostics,
        )
    } else {
        (
            SourceStatus {
                data_available: false,
                access_available: true,
                message: Some("usage data not found in CLI output".to_string()),
                cli_authorization: None,
            },
            Vec::new(),
            UsageInfo::default(),
            parsed.diagnostics,
        )
    };
    let data_as_of = live_snapshot_data_as_of(&collected_at, status.data_available);

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status,
        raw_data_available,
        collected_at: Some(collected_at),
        data_as_of,
        account: Default::default(),
        limits,
        available_limit_resets: None,
        usage,
        diagnostics,
    }
}

pub(super) fn unavailable_source_data(raw: Option<String>, message: &str) -> SourceData {
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
                cli_authorization: None,
            },
            raw_data_available,
            collected_at: Some(utc_now()),
            data_as_of: None,
            account: Default::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        },
        stderr: String::new(),
    }
}

fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn live_snapshot_data_as_of(collected_at: &str, data_available: bool) -> Option<String> {
    data_available.then(|| collected_at.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProviderRun;

    const SAMPLE_OUTPUT: &str = "\
Current session
40% used
Resets 2:20am (Asia/Nicosia)
Current week
73% used
Resets Jun 30 at 1pm (Asia/Nicosia)
Total cost: $0.0000
Usage: 0input,0output,0cacheread,0cachewrite
";

    #[test]
    fn structured_from_representative_cli_output() {
        let structured = structured_from_output(SAMPLE_OUTPUT);

        assert_eq!(structured.provider, "claude");
        assert_eq!(structured.source, "claude_cli");
        assert_eq!(structured.source_link, SOURCE_LINK);
        assert!(structured.status.access_available);
        assert!(structured.status.data_available);
        assert!(structured.status.message.is_none());
        assert!(structured.raw_data_available);
        assert!(structured.collected_at.is_some());
        assert_eq!(
            structured.data_as_of.as_deref(),
            structured.collected_at.as_deref()
        );

        assert_eq!(structured.limits.len(), 2);

        let session = &structured.limits[0];
        assert_eq!(session.name, "Current session");
        assert_eq!(session.used_percent, Some(40.0));
        assert_eq!(session.remaining_percent, Some(60.0));
        assert_eq!(session.resets_at.as_deref(), Some("2:20am (Asia/Nicosia)"));
        assert_eq!(session.window_minutes, Some(300));

        let week = &structured.limits[1];
        assert_eq!(week.name, "Current week");
        assert_eq!(week.used_percent, Some(73.0));
        assert_eq!(week.remaining_percent, Some(27.0));
        assert_eq!(week.window_minutes, Some(10080));
        assert_eq!(
            week.resets_at.as_deref(),
            Some("Jun 30 at 1pm (Asia/Nicosia)")
        );

        assert_eq!(structured.usage.money.used_amount, Some(0.0));
        assert_eq!(structured.usage.money.currency.as_deref(), Some("usd"));
        assert_eq!(structured.usage.tokens.input, Some(0));
        assert_eq!(structured.usage.tokens.output, Some(0));
        assert_eq!(structured.usage.tokens.cache_read, Some(0));
        assert_eq!(structured.usage.tokens.cache_write, Some(0));
        assert_eq!(structured.usage.tokens.total, Some(0));
    }

    #[test]
    fn structured_marks_interactive_setup_as_unavailable_data() {
        let input = "Select login method\nChoose the text style\n";
        let structured = structured_from_output(input);

        assert!(!structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.cli_authorization,
            Some(CliAuthorization::Claude)
        );
        assert_eq!(
            structured.status.message.as_deref(),
            Some("Claude CLI is installed but not authorized; run `claude login` and try again. Setup: https://code.claude.com/docs/en/setup")
        );
        assert!(structured.limits.is_empty());
    }

    #[test]
    fn unavailable_source_data_marks_cli_not_installed() {
        let data = unavailable_source_data(
            None,
            "Claude CLI is not installed or is not available in PATH; install `claude` and try again. Setup: https://code.claude.com/docs/en/setup",
        );

        assert!(!data.structured.status.access_available);
        assert!(!data.structured.status.data_available);
        assert!(!data.structured.raw_data_available);
        assert_eq!(
            data.structured.status.message.as_deref(),
            Some("Claude CLI is not installed or is not available in PATH; install `claude` and try again. Setup: https://code.claude.com/docs/en/setup")
        );
    }

    #[test]
    fn structured_marks_missing_usage_data() {
        let structured = structured_from_output("OpenAI Codex\nfor shortcuts");

        assert!(structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("usage data not found in CLI output")
        );
        assert!(structured.raw_data_available);
    }

    #[test]
    fn get_usage_returns_raw_and_structured() {
        let run = ProviderRun {
            compacted_stdout: SAMPLE_OUTPUT.to_string(),
            stderr: String::new(),
        };

        let data = build_source_data(&run);

        assert_eq!(data.raw.as_deref(), Some(SAMPLE_OUTPUT));
        assert!(data.structured.status.data_available);
        assert!(!data.structured.limits.is_empty());
    }

    #[test]
    fn build_source_data_preserves_raw_stdout_and_stderr_diagnostics() {
        let run = ProviderRun {
            compacted_stdout: SAMPLE_OUTPUT.to_string(),
            stderr: "expect warning\n".to_string(),
        };

        let data = build_source_data(&run);

        assert_eq!(data.raw.as_deref(), Some(SAMPLE_OUTPUT));
        assert_eq!(data.stderr, "expect warning\n");
        assert!(data.structured.status.data_available);
        assert!(data
            .structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("stderr: expect warning")));
    }
}
