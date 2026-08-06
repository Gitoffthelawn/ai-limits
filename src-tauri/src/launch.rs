use tauri::{AppHandle, Manager};

use crate::windows::MAIN_WINDOW_LABEL;

#[cfg(target_os = "macos")]
const FIRST_LAUNCH_MARKER_FILE: &str = "main-window-opened";

/// Returns whether this is the first launch whose Main Window should be opened.
///
/// The marker is created atomically in Tauri's application-data directory. If
/// the directory cannot be created or written, returning `true` preserves an
/// accessible interface instead of starting the application with no visible
/// window.
#[cfg(target_os = "macos")]
pub fn should_open_main_window(app: &AppHandle) -> bool {
    use std::fs::{self, OpenOptions};

    let Ok(data_dir) = app.path().app_data_dir() else {
        return true;
    };
    if fs::create_dir_all(&data_dir).is_err() {
        return true;
    }

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(data_dir.join(FIRST_LAUNCH_MARKER_FILE))
    {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
        Err(_) => true,
    }
}

/// Shows the Main Window on macOS only when this is the first launch. Other
/// desktop platforms have no Menu Bar Popover and retain their existing
/// Main-Window-at-launch behavior.
pub fn show_initial_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let should_show = should_open_main_window(app);
    #[cfg(not(target_os = "macos"))]
    let should_show = true;

    if should_show {
        if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
            let _ = main_window.show();
        }
    }
}
