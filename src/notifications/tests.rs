use super::*;
use crate::types::{AccountInfo, LimitInfo, Source, SourceData, SourceStatus, UsageInfo};
use std::cell::Cell;

fn structured_with_limit(remaining_percent: Option<f64>) -> StructuredSourceInfo {
    StructuredSourceInfo {
        provider: "codex".to_string(),
        source: "codex_local".to_string(),
        source_link: String::new(),
        status: SourceStatus {
            data_available: true,
            access_available: true,
            message: None,
            cli_authorization: None,
        },
        raw_data_available: false,
        collected_at: None,
        data_as_of: None,
        account: AccountInfo::default(),
        limits: vec![LimitInfo {
            name: "5h".to_string(),
            remaining_percent,
            ..Default::default()
        }],
        available_limit_resets: None,
        usage: UsageInfo::default(),
        diagnostics: Vec::new(),
    }
}

#[test]
fn creates_notification_for_threshold_remaining_percent() {
    let notifications = notifications_for_structured(&structured_with_limit(Some(75.0)));

    assert_eq!(
        notifications,
        vec![Notification {
            dedupe_key: "Codex|codex_local|5h|75".to_string(),
            title: "🟢 AI Limits".to_string(),
            subtitle: "Codex 5h - 75% left".to_string(),
            message: "reset unknown".to_string(),
            color: NotificationColor::Green,
        }]
    );
}

#[test]
fn creates_notification_when_remaining_is_below_threshold() {
    let notifications = notifications_for_structured(&structured_with_limit(Some(74.0)));

    assert_eq!(notifications[0].color, NotificationColor::Green);
    assert_eq!(notifications[0].title, "🟢 AI Limits");
    assert_eq!(notifications[0].subtitle, "Codex 5h - 74% left");
    assert_eq!(notifications[0].message, "reset unknown");
}

#[test]
fn dedupe_key_uses_threshold_not_exact_remaining_percent() {
    let first = notifications_for_structured(&structured_with_limit(Some(75.0)));
    let second = notifications_for_structured(&structured_with_limit(Some(74.0)));

    assert_eq!(first[0].dedupe_key, second[0].dedupe_key);
}

#[test]
fn ignores_remaining_above_first_threshold() {
    assert!(notifications_for_structured(&structured_with_limit(Some(76.0))).is_empty());
}

#[test]
fn derives_remaining_percent_from_used_percent() {
    let mut info = structured_with_limit(None);
    info.limits[0].used_percent = Some(50.0);

    let notifications = notifications_for_structured(&info);

    assert_eq!(notifications[0].color, NotificationColor::Yellow);
    assert_eq!(notifications[0].title, "🟡 AI Limits");
    assert_eq!(notifications[0].subtitle, "Codex 5h - 50% left");
}

#[test]
fn formats_notification_reset_with_shared_time_display() {
    let mut info = structured_with_limit(Some(50.0));
    info.collected_at = Some("2026-06-30T20:00:00Z".to_string());
    info.limits[0].resets_at = Some("2026-06-30T20:41:00Z".to_string());

    let notifications = notifications_for_structured(&info);

    assert!(notifications[0].message.starts_with("reset "));
    assert_ne!(notifications[0].message, "reset unknown");
    assert!(!notifications[0].message.contains("UTC"));
    assert!(!notifications[0].message.contains('T'));
    assert!(!notifications[0].message.ends_with('Z'));
}

#[test]
fn ignores_unavailable_data() {
    let mut info = structured_with_limit(Some(25.0));
    info.status.data_available = false;

    assert!(notifications_for_structured(&info).is_empty());
}

#[test]
fn send_for_report_dedupes_within_session() {
    struct CountingDelivery(Cell<usize>);

    impl NotificationDelivery for CountingDelivery {
        fn deliver(&self, _notification: &Notification) -> io::Result<()> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }

    let report = SourceReport {
        source: Source::CodexLocal,
        data: SourceData {
            raw: None,
            structured: structured_with_limit(Some(75.0)),
            stderr: String::new(),
        },
    };
    let mut sent = HashSet::new();
    let delivery = CountingDelivery(Cell::new(0));

    send_for_report_with_delivery(&report, &mut sent, &delivery);
    assert_eq!(sent.len(), 1);
    assert_eq!(delivery.0.get(), 1);

    send_for_report_with_delivery(&report, &mut sent, &delivery);
    assert_eq!(sent.len(), 1);
    assert_eq!(delivery.0.get(), 1);
}

#[test]
fn evaluates_source_report_structured_data() {
    let report = SourceReport {
        source: Source::CodexLocal,
        data: SourceData {
            raw: None,
            structured: structured_with_limit(Some(10.0)),
            stderr: String::new(),
        },
    };

    assert_eq!(
        notifications_for_report(&report)[0].color,
        NotificationColor::Red
    );
}
