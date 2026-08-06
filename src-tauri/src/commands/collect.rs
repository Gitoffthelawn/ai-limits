use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use ai_limits::get_limits::{
    get_source_plan_limits, ui_source_plan, SourcePlan, UiSourcePlanOptions,
};
use ai_limits::notifications as core_notifications;
use ai_limits::notifications::PreviousRemainingStore;
use ai_limits::types::{SourceReport, StructuredSourceInfo};

use super::provider_limits::{
    provider_error, provider_limits_from_structured, ProviderLimits, ProviderLimitsQuery,
};
use super::structured_cache::{CollectionCoordinator, StructuredInfoCache};

pub(super) async fn collect_single_provider_limits(
    provider_id: &str,
    query: &ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
    remaining_store: Arc<dyn PreviousRemainingStore>,
    structured_cache: StructuredInfoCache,
    coordinator: CollectionCoordinator,
) -> Result<ProviderLimits, String> {
    let source_plan = ui_source_plan(source_plan_options(query))
        .into_iter()
        .find(|plan| plan.label() == provider_id)
        .ok_or_else(|| format!("Provider '{provider_id}' is disabled or unknown"))?;

    let id = source_plan.label().to_string();
    let notifications_enabled = query.notifications_enabled;

    let result = coordinator
        .collect_once(&id, move || {
            run_collection(
                source_plan,
                notifications_enabled,
                app,
                sent_notifications,
                remaining_store,
                structured_cache,
            )
        })
        .await;

    Ok(match result {
        Ok(structured) => provider_limits_from_structured(&id, &structured),
        Err(error) => provider_error(&id, error),
    })
}

/// The single actual collection for one provider: runs the source chain,
/// evaluates notifications once on success, and — only on success — updates
/// the shared structured-data cache. Always runs on a blocking thread, since
/// the source chain performs blocking file/process/network I/O.
async fn run_collection(
    source_plan: SourcePlan,
    notifications_enabled: bool,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
    remaining_store: Arc<dyn PreviousRemainingStore>,
    structured_cache: StructuredInfoCache,
) -> Result<StructuredSourceInfo, String> {
    let id = source_plan.label().to_string();

    tauri::async_runtime::spawn_blocking(move || match get_source_plan_limits(source_plan) {
        Ok(report) => {
            if notifications_enabled {
                notify_for_report(&report, app, &sent_notifications, &remaining_store);
            }
            let structured = report.data.structured;
            if let Ok(mut cache) = structured_cache.lock() {
                cache.insert(id, structured.clone());
            }
            Ok(structured)
        }
        Err(error) => Err(error.to_string()),
    })
    .await
    .map_err(|error| error.to_string())?
}

fn source_plan_options(query: &ProviderLimitsQuery) -> UiSourcePlanOptions {
    UiSourcePlanOptions {
        enabled_codex: query.enabled_codex,
        enabled_claude: query.enabled_claude,
        enabled_cursor: query.enabled_cursor,
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
