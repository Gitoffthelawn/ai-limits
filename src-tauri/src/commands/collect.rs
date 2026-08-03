use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use ai_limits::get_limits::{
    get_source_plan_limits, ui_source_plan, SourcePlan, UiSourcePlanOptions,
};
use ai_limits::notifications as core_notifications;
use ai_limits::notifications::PreviousRemainingStore;
use ai_limits::types::SourceReport;

use super::provider_limits::{
    provider_error, provider_limits_from_structured, ProviderLimits, ProviderLimitsQuery,
};

pub(super) fn collect_single_provider_limits(
    provider_id: &str,
    query: &ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
    remaining_store: Arc<dyn PreviousRemainingStore>,
) -> Result<ProviderLimits, String> {
    let source_plan = ui_source_plan(source_plan_options(query))
        .into_iter()
        .find(|plan| plan.label() == provider_id)
        .ok_or_else(|| format!("Provider '{provider_id}' is disabled or unknown"))?;

    Ok(collect_provider_limits_for_plan(
        source_plan,
        query,
        app,
        sent_notifications,
        remaining_store,
    ))
}

fn collect_provider_limits_for_plan(
    source_plan: SourcePlan,
    query: &ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
    remaining_store: Arc<dyn PreviousRemainingStore>,
) -> ProviderLimits {
    let id = source_plan.label().to_string();
    match get_source_plan_limits(source_plan) {
        Ok(report) => {
            if query.notifications_enabled {
                notify_for_report(&report, app, &sent_notifications, &remaining_store);
            }
            provider_limits_from_structured(&id, &report.data.structured)
        }
        Err(error) => provider_error(&id, error.to_string()),
    }
}

fn source_plan_options(query: &ProviderLimitsQuery) -> UiSourcePlanOptions {
    UiSourcePlanOptions {
        enabled_codex: query.enabled_codex,
        enabled_claude: query.enabled_claude,
        enabled_cursor: query.enabled_cursor,
        source_priority: query.source_priority,
    }
}

fn notify_for_report(
    report: &SourceReport,
    app: tauri::AppHandle,
    sent_notifications: &Arc<Mutex<HashSet<String>>>,
    remaining_store: &Arc<dyn PreviousRemainingStore>,
) {
    let Ok(mut sent) = sent_notifications.lock() else {
        return;
    };

    let delivery = crate::notifications::TauriNotificationDelivery::new(app);
    core_notifications::send_for_report_with_delivery(
        report,
        &mut sent,
        remaining_store.as_ref(),
        &delivery,
    );
}
