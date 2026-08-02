//! Silent update download for the desktop app.
//!
//! The frontend owns the schedule and the user's automatic-update setting; this
//! module only performs one check-and-download on request. A downloaded update
//! is installed in place but never applied by force: the running application
//! keeps its current version until the user restarts it.

use serde::Serialize;

/// A staged update, reported back so the frontend can offer a restart.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StagedUpdate {
    pub version: String,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn download_app_update(app: tauri::AppHandle) -> Result<Option<StagedUpdate>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|error| error.to_string())?;

    let Some(update) = updater.check().await.map_err(|error| error.to_string())? else {
        return Ok(None);
    };

    let staged = StagedUpdate {
        version: update.version.clone(),
        notes: update.body.clone(),
    };

    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|error| error.to_string())?;

    Ok(Some(staged))
}

/// Restarts the application so a staged update takes effect. Only ever called
/// after the user chooses to restart.
#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) -> Result<(), String> {
    app.restart()
}
