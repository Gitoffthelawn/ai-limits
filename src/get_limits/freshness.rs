use chrono::{DateTime, Duration, Utc};

use crate::types::{Source, SourceReport};

const LOCAL_RESET_EXPIRY_GRACE_MINUTES: i64 = 2;
pub(super) const STALE_LOCAL_DATA_MESSAGE: &str = "Local provider data is outdated";

pub(super) fn mark_expired_local_limit_data(
    mut report: SourceReport,
    now: DateTime<Utc>,
) -> SourceReport {
    if !matches!(report.source, Source::CodexLocal | Source::ClaudeLocal) {
        return report;
    }

    let expiry_cutoff = now - Duration::minutes(LOCAL_RESET_EXPIRY_GRACE_MINUTES);
    let has_expired_reset = report.data.structured.limits.iter().any(|limit| {
        limit
            .resets_at
            .as_deref()
            .and_then(parse_absolute_reset)
            .is_some_and(|reset| reset < expiry_cutoff)
    });
    if !has_expired_reset {
        return report;
    }

    report.data.structured.status.data_available = false;
    report.data.structured.status.message = Some(STALE_LOCAL_DATA_MESSAGE.to_string());
    report.data.structured.limits.clear();
    report.data.structured.available_limit_resets = None;
    report.data.structured.diagnostics.push(
        "local limit snapshot rejected because an automatic reset time is in the past".to_string(),
    );
    report
}

fn parse_absolute_reset(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LimitInfo, SourceData, SourceStatus, StructuredSourceInfo};

    fn report_for(
        source: Source,
        access_available: bool,
        data_available: bool,
        limits: Vec<LimitInfo>,
    ) -> SourceReport {
        SourceReport {
            source,
            data: SourceData {
                raw: None,
                structured: StructuredSourceInfo {
                    provider: "codex".to_string(),
                    source: "codex_local".to_string(),
                    source_link: String::new(),
                    status: SourceStatus {
                        access_available,
                        data_available,
                        message: None,
                        cli_authorization: None,
                    },
                    raw_data_available: false,
                    collected_at: None,
                    data_as_of: None,
                    account: Default::default(),
                    limits,
                    available_limit_resets: None,
                    usage: Default::default(),
                    diagnostics: Vec::new(),
                },
                stderr: String::new(),
            },
        }
    }

    fn has_usable_limit_data(report: &SourceReport) -> bool {
        report.data.structured.status.access_available
            && report.data.structured.status.data_available
            && !report.data.structured.limits.is_empty()
    }

    #[test]
    fn expired_codex_local_reset_rejects_the_whole_limit_snapshot() {
        let report = report_for(
            Source::CodexLocal,
            true,
            true,
            vec![
                LimitInfo {
                    resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                    ..Default::default()
                },
                LimitInfo {
                    resets_at: Some("2026-08-01T09:00:00Z".to_string()),
                    ..Default::default()
                },
            ],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T09:03:00Z".parse().expect("valid timestamp"),
        );

        assert!(!result.data.structured.status.data_available);
        assert_eq!(
            result.data.structured.status.message.as_deref(),
            Some(STALE_LOCAL_DATA_MESSAGE)
        );
        assert!(result.data.structured.limits.is_empty());
        assert!(!has_usable_limit_data(&result));
    }

    #[test]
    fn expired_claude_local_reset_is_rejected_even_if_provider_reconstruction_returns_it() {
        let report = report_for(
            Source::ClaudeLocal,
            true,
            true,
            vec![LimitInfo {
                resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                ..Default::default()
            }],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T09:03:00Z".parse().expect("valid timestamp"),
        );

        assert!(!result.data.structured.status.data_available);
        assert_eq!(
            result.data.structured.status.message.as_deref(),
            Some(STALE_LOCAL_DATA_MESSAGE)
        );
        assert!(result.data.structured.limits.is_empty());
    }

    #[test]
    fn local_reset_within_clock_grace_remains_usable() {
        let report = report_for(
            Source::ClaudeLocal,
            true,
            true,
            vec![LimitInfo {
                resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                ..Default::default()
            }],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T09:01:59Z".parse().expect("valid timestamp"),
        );

        assert!(has_usable_limit_data(&result));
    }

    #[test]
    fn non_local_sources_are_not_rejected_by_local_freshness_rule() {
        let report = report_for(
            Source::CodexCli,
            true,
            true,
            vec![LimitInfo {
                resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                ..Default::default()
            }],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T10:00:00Z".parse().expect("valid timestamp"),
        );

        assert!(has_usable_limit_data(&result));
    }
}
