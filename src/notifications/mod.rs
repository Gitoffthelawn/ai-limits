use std::collections::HashSet;
use std::io;

use crate::presentation::TimeContext;
use crate::types::{SourceReport, StructuredSourceInfo};

mod content;
mod kinds;
mod tauri_bridge;

pub use content::Notification;
pub use kinds::{LimitNotificationKind, NotificationColor};

pub const TAURI_NOTIFICATION_BRIDGE_ADDR: &str = tauri_bridge::NOTIFICATION_BRIDGE_ADDR;

pub fn notify(notification: &Notification) -> io::Result<()> {
    tauri_bridge::TauriNotificationBridge.deliver(notification)
}

pub fn notify_test(kind: LimitNotificationKind) -> io::Result<()> {
    notify(&Notification::test(kind))
}

pub trait NotificationDelivery {
    fn deliver(&self, notification: &Notification) -> io::Result<()>;
}

pub fn send_for_report(report: &SourceReport, sent: &mut HashSet<String>) {
    send_for_report_with_delivery(report, sent, &tauri_bridge::TauriNotificationBridge);
}

pub fn send_for_report_with_delivery(
    report: &SourceReport,
    sent: &mut HashSet<String>,
    delivery: &impl NotificationDelivery,
) {
    for notification in notifications_for_report(report) {
        if sent.insert(notification.dedupe_key.clone()) {
            let _ = delivery.deliver(&notification);
        }
    }
}

pub fn notifications_for_report(report: &SourceReport) -> Vec<Notification> {
    notifications_for_structured(&report.data.structured)
}

pub fn notifications_for_structured(info: &StructuredSourceInfo) -> Vec<Notification> {
    if !info.status.access_available || !info.status.data_available {
        return Vec::new();
    }

    let time_context = TimeContext::from_structured(info);

    info.limits
        .iter()
        .filter_map(|limit| {
            let remaining = kinds::remaining_percent(limit)?;
            let kind = kinds::matching_kind(remaining)?;
            Some(Notification::limit(
                &info.provider,
                &info.source,
                &limit.name,
                kind,
                remaining,
                limit.resets_at.as_deref(),
                &time_context,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests;
