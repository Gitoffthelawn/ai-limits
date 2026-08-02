mod commands;
mod notifications;
mod platform;

use std::collections::HashSet;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

/// Help sub-pages exposed in the macOS native Help menu. Kept in sync with the
/// `HELP_CHAPTERS` list in the frontend (frontend/modules/help-chapters.js) so
/// the native menu mirrors the in-app Help sidebar.
#[cfg(target_os = "macos")]
const HELP_CHAPTERS: &[(&str, &str)] = &[
    ("about", "About"),
    ("providers", "Providers"),
    ("source-priority", "Source priority"),
    ("data-errors", "Data availability"),
    ("notifications", "Notifications"),
    ("updates", "Updates"),
    ("permissions", "Permissions"),
    ("cli-mode", "CLI mode"),
    ("limitations", "Limitations"),
    ("for-developers", "For developers"),
];

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--cli") {
        if ai_limits::cli::run_with_args(std::env::args().skip(2)) == ExitCode::FAILURE {
            std::process::exit(1);
        }
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Arc::new(Mutex::new(HashSet::<String>::new())))
        .setup(|app| {
            notifications::start_notification_bridge(app.handle().clone());
            #[cfg(target_os = "macos")]
            install_help_menu(app)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_single_provider_limits,
            commands::open_external_url,
            commands::start_provider_cli_login,
            commands::get_cli_command,
            commands::run_cli_in_terminal,
            commands::app_update::download_app_update,
            commands::app_update::restart_app,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}

/// Appends the Help sub-pages to the default macOS Help menu so each chapter can
/// be opened from the menu bar in addition to the in-app info button.
#[cfg(target_os = "macos")]
fn install_help_menu(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, HELP_SUBMENU_ID};

    let handle = app.handle();
    let menu = Menu::default(handle)?;

    if let Some(help_menu) = menu
        .get(HELP_SUBMENU_ID)
        .and_then(|item| item.as_submenu().cloned())
    {
        for (id, label) in HELP_CHAPTERS {
            let item = MenuItem::with_id(handle, format!("help:{id}"), label, true, None::<&str>)?;
            help_menu.append(&item)?;
        }
    }

    app.set_menu(menu)?;
    Ok(())
}

/// Routes a native Help menu selection to the web view, which owns the Help UI.
fn handle_menu_event(app: &tauri::AppHandle, menu_id: &str) {
    use tauri::Manager;

    let Some(chapter) = menu_id.strip_prefix("help:") else {
        return;
    };
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let chapter = serde_json::to_string(chapter).unwrap_or_else(|_| "\"\"".to_string());
    let _ = window.eval(format!(
        "window.__openHelpFromNative && window.__openHelpFromNative({chapter})"
    ));
}
