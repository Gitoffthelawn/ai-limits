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
            commands::get_app_version,
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

/// Builds the macOS menu bar. Starts from the app/Edit/Window/Help submenus
/// Tauri would generate by default, but drops File and View (unused by this
/// app) and replaces the app-menu Services item with a Settings item that
/// opens the in-app settings panel.
#[cfg(target_os = "macos")]
fn install_help_menu(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID};

    let handle = app.handle();
    let pkg_info = handle.package_info();
    let about_metadata = tauri::menu::AboutMetadata {
        name: Some(pkg_info.name.clone()),
        version: Some(pkg_info.version.to_string()),
        ..Default::default()
    };

    let settings_item = MenuItem::with_id(
        handle,
        "open-settings",
        "Settings…",
        true,
        Some("Cmd+,"),
    )?;

    let app_menu = Submenu::with_items(
        handle,
        pkg_info.name.clone(),
        true,
        &[
            &PredefinedMenuItem::about(handle, None, Some(about_metadata))?,
            &PredefinedMenuItem::separator(handle)?,
            &settings_item,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::hide(handle, None)?,
            &PredefinedMenuItem::hide_others(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::quit(handle, None)?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        handle,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(handle, None)?,
            &PredefinedMenuItem::redo(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::cut(handle, None)?,
            &PredefinedMenuItem::copy(handle, None)?,
            &PredefinedMenuItem::paste(handle, None)?,
            &PredefinedMenuItem::select_all(handle, None)?,
        ],
    )?;

    let window_menu = Submenu::with_id_and_items(
        handle,
        WINDOW_SUBMENU_ID,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(handle, None)?,
            &PredefinedMenuItem::maximize(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::close_window(handle, None)?,
        ],
    )?;

    let help_menu = Submenu::with_id_and_items(handle, HELP_SUBMENU_ID, "Help", true, &[])?;
    for (id, label) in HELP_CHAPTERS {
        let item = MenuItem::with_id(handle, format!("help:{id}"), label, true, None::<&str>)?;
        help_menu.append(&item)?;
    }

    let menu = Menu::with_items(
        handle,
        &[&app_menu, &edit_menu, &window_menu, &help_menu],
    )?;

    app.set_menu(menu)?;
    Ok(())
}

/// Routes native app-menu selections to the web view, which owns the Help and
/// Settings UI.
fn handle_menu_event(app: &tauri::AppHandle, menu_id: &str) {
    use tauri::Manager;

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if menu_id == "open-settings" {
        let _ = window.eval("window.__openSettingsFromNative && window.__openSettingsFromNative()");
        return;
    }

    let Some(chapter) = menu_id.strip_prefix("help:") else {
        return;
    };

    let chapter = serde_json::to_string(chapter).unwrap_or_else(|_| "\"\"".to_string());
    let _ = window.eval(format!(
        "window.__openHelpFromNative && window.__openHelpFromNative({chapter})"
    ));
}
