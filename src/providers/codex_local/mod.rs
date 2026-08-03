mod auth;
mod parse;
mod project;
mod raw;
mod scan;

use chrono::Utc;

use crate::types::SourceData;

pub use project::decode_raw;
pub use raw::{
    CodexLocalRateLimitWindow, CodexLocalRateLimits, CodexLocalRaw, CodexLocalTokenTotals,
};

use auth::{read_subscription, CodexLocalSubscription};
use project::{build_structured, source_data_from_raw};
use raw::{raw_from_usage, CodexLocalUsage};
use scan::{codex_home, scan_dir};

pub fn get_usage() -> std::io::Result<SourceData> {
    collect()
}

pub fn collect() -> std::io::Result<SourceData> {
    let root = codex_home()?;
    let now = Utc::now();
    let collected_at = Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string());

    if !root.exists() {
        let raw = CodexLocalRaw {
            root: root.display().to_string(),
            ..CodexLocalRaw::default()
        };
        let structured = build_structured(
            &raw,
            &CodexLocalSubscription::default(),
            collected_at,
            false,
            false,
            Some(format!("not found: {}", root.display())),
        );
        return Ok(source_data_from_raw(&raw, structured));
    }

    let mut usage = CodexLocalUsage::default();
    scan_dir(&root.join("sessions"), &mut usage)?;
    scan_dir(&root.join("archived_sessions"), &mut usage)?;

    let raw = raw_from_usage(&root, &usage);
    let (data_available, message) = if usage.token_events == 0 {
        (false, Some("token events: not found".to_string()))
    } else {
        (true, None)
    };
    let subscription = read_subscription(&root, now);
    let structured = build_structured(
        &raw,
        &subscription,
        collected_at,
        true,
        data_available,
        message,
    );

    Ok(source_data_from_raw(&raw, structured))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::auth::CodexLocalSubscription;
    use super::parse::parse_credits;
    use super::project::{build_structured, decode_raw, format_unix_utc};
    use super::raw::{raw_from_usage, CodexLocalRateLimits, CodexLocalRaw, CodexLocalUsage};
    use super::scan::scan_file;
    use crate::types::{SourceData, StructuredSourceInfo};

    fn structured_for(
        raw: &CodexLocalRaw,
        access_available: bool,
        data_available: bool,
        message: Option<&str>,
    ) -> StructuredSourceInfo {
        build_structured(
            raw,
            &CodexLocalSubscription::default(),
            None,
            access_available,
            data_available,
            message.map(ToString::to_string),
        )
    }

    fn fixture_path(suffix: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "ai-limits-codex-local-{}-{suffix}.jsonl",
            std::process::id()
        ))
    }

    fn scan_fixture(content: &str, suffix: &str) -> CodexLocalRaw {
        let path = fixture_path(suffix);
        fs::write(&path, content).expect("write fixture");

        let mut usage = CodexLocalUsage::default();
        scan_file(&path, &mut usage).expect("scan fixture");
        let raw = raw_from_usage(Path::new("/tmp/.codex"), &usage);
        let _ = fs::remove_file(&path);
        raw
    }

    #[test]
    fn builds_structured_data_from_representative_sample() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T10:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":2,"output_tokens":3,"reasoning_output_tokens":1,"total_tokens":16}},"rate_limits":{"primary":{"used_percent":12.4,"window_minutes":300,"resets_at":1750000000}}}}
{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":"45","window_minutes":"300","resets_at":"1750003600"},"secondary":{"used_percent":71.9,"window_minutes":10080,"resets_at":1750600000},"credits":123.6,"plan_type":"pro"}}}
"#,
            "structured",
        );

        let structured = structured_for(&raw, true, true, None);

        assert_eq!(structured.provider, "codex");
        assert_eq!(structured.source, "codex_local");
        assert_eq!(structured.source_link, "docs/get-limits");
        assert!(structured.status.access_available);
        assert!(structured.status.data_available);
        assert!(structured.raw_data_available);
        assert_eq!(raw.token_events, 2);
        assert_eq!(structured.usage.tokens.input, Some(30));
        assert_eq!(structured.usage.tokens.cached_input, Some(6));
        assert_eq!(structured.usage.tokens.output, Some(9));
        assert_eq!(structured.usage.tokens.reasoning_output, Some(3));
        assert_eq!(structured.usage.tokens.total, Some(48));
        assert_eq!(structured.usage.activity.events_count, Some(2));
        assert_eq!(structured.usage.activity.files_count, Some(1));
        assert_eq!(
            structured.usage.activity.latest_activity_at.as_deref(),
            Some("2026-06-28T11:00:00Z")
        );
        assert_eq!(
            structured.data_as_of.as_deref(),
            Some("2026-06-28T11:00:00Z")
        );
        assert_eq!(structured.account.plan.as_deref(), Some("pro"));
        assert_eq!(structured.account.credits_remaining, Some(123.6));

        let primary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "primary")
            .expect("primary limit");
        assert_eq!(primary.used_percent, Some(45.0));
        assert_eq!(primary.remaining_percent, Some(55.0));
        assert_eq!(primary.window_minutes, Some(300));
        assert_eq!(primary.window_label.as_deref(), Some("5h (300m)"));
        assert_eq!(primary.resets_at.as_deref(), Some("2025-06-15T16:06:40Z"));

        let secondary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "secondary")
            .expect("secondary limit");
        assert_eq!(secondary.used_percent, Some(71.9));
        assert_eq!(secondary.remaining_percent, Some(28.1));
        assert_eq!(secondary.window_label.as_deref(), Some("weekly (10080m)"));
    }

    #[test]
    fn structured_marks_missing_root_as_inaccessible() {
        let raw = CodexLocalRaw {
            root: "/missing/.codex".to_string(),
            ..CodexLocalRaw::default()
        };
        let structured = structured_for(&raw, false, false, Some("not found: /missing/.codex"));

        assert!(!structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("not found: /missing/.codex")
        );
        assert!(structured.limits.is_empty());
        assert_eq!(structured.usage.tokens.total, None);
    }

    #[test]
    fn structured_marks_accessible_root_without_token_events() {
        let raw = CodexLocalRaw {
            root: "/tmp/.codex".to_string(),
            files_scanned: 3,
            ..CodexLocalRaw::default()
        };
        let structured = structured_for(&raw, true, false, Some("token events: not found"));

        assert!(structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("token events: not found")
        );
        assert_eq!(structured.usage.activity.files_count, Some(3));
        assert_eq!(structured.usage.activity.events_count, Some(0));
    }

    #[test]
    fn keeps_limits_from_latest_rate_limits_event_not_latest_event() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T10:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5,"reasoning_output_tokens":0,"total_tokens":15}},"rate_limits":{"primary":{"used_percent":10,"window_minutes":300,"resets_at":1750000000}}}}
{"type":"event_msg","timestamp":"2026-06-28T12:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":5,"cached_input_tokens":0,"output_tokens":2,"reasoning_output_tokens":0,"total_tokens":7}}}}
"#,
            "latest-limits",
        );

        let structured = structured_for(&raw, true, true, None);
        let primary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "primary")
            .expect("primary limit");

        assert_eq!(primary.used_percent, Some(10.0));
        assert_eq!(
            structured.usage.activity.latest_activity_at.as_deref(),
            Some("2026-06-28T10:00:00Z")
        );
        assert_eq!(raw.totals.total_tokens, 22);
    }

    #[test]
    fn accepts_rate_limits_only_token_count_with_null_info() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-29T01:46:39.473Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5,"reasoning_output_tokens":0,"total_tokens":15}},"rate_limits":{"primary":{"used_percent":86.0,"window_minutes":300,"resets_at":1782709162},"plan_type":"plus"}}}
{"type":"event_msg","timestamp":"2026-06-29T02:24:02.237Z","payload":{"type":"token_count","info":null,"rate_limits":{"primary":{"used_percent":100.0,"window_minutes":300,"resets_at":1782709162},"secondary":{"used_percent":16.0,"window_minutes":10080,"resets_at":1783295962},"credits":{"has_credits":true,"unlimited":false,"balance":"336.2474587500"},"plan_type":"plus"}}}
"#,
            "null-info",
        );

        let structured = structured_for(&raw, true, true, None);

        assert_eq!(raw.totals.total_tokens, 15);
        assert_eq!(structured.account.plan.as_deref(), Some("plus"));
        assert_eq!(structured.account.credits_remaining, Some(336.2474587500));

        let primary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "primary")
            .expect("primary limit");
        assert_eq!(primary.used_percent, Some(100.0));
        assert_eq!(primary.remaining_percent, Some(0.0));
    }

    #[test]
    fn unlimited_credits_adds_diagnostic_and_null_remaining() {
        let raw = CodexLocalRaw {
            root: "/tmp/.codex".to_string(),
            token_events: 1,
            latest_rate_limits: Some(CodexLocalRateLimits {
                credits_unlimited: true,
                plan_type: Some("pro".to_string()),
                ..CodexLocalRateLimits::default()
            }),
            ..CodexLocalRaw::default()
        };

        let structured = structured_for(&raw, true, true, None);

        assert_eq!(structured.account.credits_remaining, None);
        assert!(structured
            .diagnostics
            .contains(&"credits: unlimited".to_string()));
    }

    #[test]
    fn subscription_dates_reach_account_with_renewal_note() {
        let raw = CodexLocalRaw {
            root: "/tmp/.codex".to_string(),
            token_events: 1,
            ..CodexLocalRaw::default()
        };
        let subscription = CodexLocalSubscription {
            started_at: Some("2026-06-02T11:47:15Z".to_string()),
            renewal_at: Some("2026-08-02T11:47:15Z".to_string()),
            diagnostics: vec!["renewal date is the subscription active-until date".to_string()],
        };

        let structured = build_structured(&raw, &subscription, None, true, true, None);

        assert_eq!(
            structured.account.subscription_started_at.as_deref(),
            Some("2026-06-02T11:47:15Z")
        );
        assert_eq!(
            structured.account.renewal_at.as_deref(),
            Some("2026-08-02T11:47:15Z")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("active-until")));
        assert_eq!(structured.account.price_amount, None);
        assert_eq!(structured.account.price_currency, None);
        assert_eq!(structured.account.plan_management_url, None);
        assert_eq!(structured.account.billing_management_url, None);
    }

    #[test]
    fn missing_subscription_dates_stay_null() {
        let raw = CodexLocalRaw {
            root: "/tmp/.codex".to_string(),
            token_events: 1,
            ..CodexLocalRaw::default()
        };

        let structured = structured_for(&raw, true, true, None);

        assert_eq!(structured.account.subscription_started_at, None);
        assert_eq!(structured.account.renewal_at, None);
    }

    #[test]
    fn parses_unlimited_credits_object() {
        assert_eq!(
            parse_credits(Some(&serde_json::json!({
                "has_credits": true,
                "unlimited": true
            }))),
            (None, true)
        );
        assert_eq!(
            parse_credits(Some(&serde_json::json!({
                "has_credits": true,
                "unlimited": false,
                "balance": "336.2474587500"
            }))),
            (Some(336.2474587500), false)
        );
    }

    #[test]
    fn formats_unix_seconds_as_utc_rfc3339() {
        assert_eq!(format_unix_utc(1_782_709_162), "2026-06-29T04:59:22Z");
        assert_eq!(format_unix_utc(1_750_003_600), "2025-06-15T16:06:40Z");
    }

    #[test]
    fn decode_raw_parses_serialized_usage() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":45,"window_minutes":300,"resets_at":1750003600},"plan_type":"pro"}}}
"#,
            "decode-raw",
        );

        let json = serde_json::to_string(&raw).expect("serialize raw");
        assert_eq!(decode_raw(Some(&json)), Some(raw));
    }

    #[test]
    fn collect_returns_source_data_with_json_raw() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":45,"window_minutes":300,"resets_at":1750003600},"plan_type":"pro"}}}
"#,
            "collect-source-data",
        );
        let structured = structured_for(&raw, true, true, None);
        let data = SourceData {
            raw: serde_json::to_string(&raw).ok(),
            structured,
            stderr: String::new(),
        };

        assert_eq!(decode_raw(data.raw.as_deref()), Some(raw));
        assert!(data.structured.status.data_available);
    }

    #[test]
    fn raw_serializes_to_json() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":45,"window_minutes":300,"resets_at":1750003600},"plan_type":"pro"}}}
"#,
            "raw-json",
        );

        let json = serde_json::to_value(&raw).expect("serialize raw");
        assert_eq!(json["token_events"], 1);
        assert_eq!(json["totals"]["total_tokens"], 32);
        assert_eq!(json["latest_rate_limits"]["plan_type"], "pro");
    }
}
