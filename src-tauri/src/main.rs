mod commands;
mod notifications;
mod platform;
mod windows;

use std::collections::HashSet;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::windows::MAIN_WINDOW_LABEL;
#[cfg(target_os = "macos")]
use crate::windows::{
    POPOVER_CORNER_RADIUS, POPOVER_DEFAULT_HEIGHT, POPOVER_MENU_BAR_GAP, POPOVER_SCREEN_MARGIN,
    POPOVER_WIDTH, POPOVER_WINDOW_LABEL,
};

/// Menu item id opening (showing + focusing) the Main Window.
#[cfg(target_os = "macos")]
const MENU_ID_OPEN_MAIN_WINDOW: &str = "open-main-window";

/// Menu item id opening the Main Window on its Settings panel.
const MENU_ID_OPEN_SETTINGS: &str = "open-settings";

/// Help sub-pages exposed in the macOS native Help menu. Kept in sync with the
/// `HELP_CHAPTERS` list in the frontend (frontend/modules/help-chapters.js) so
/// the native menu mirrors the in-app Help sidebar.
#[cfg(target_os = "macos")]
const HELP_CHAPTERS: &[(&str, &str)] = &[
    ("about", "About"),
    ("providers", "Providers"),
    ("source", "Source"),
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

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Arc::new(Mutex::new(HashSet::<String>::new())))
        .setup(|app| {
            notifications::start_notification_bridge(app.handle().clone());
            let store_path = notifications::previous_remaining_store_path(app.handle())?;
            let remaining_store: Arc<dyn ai_limits::notifications::PreviousRemainingStore> =
                Arc::new(ai_limits::notifications::FileRemainingStore::new(
                    store_path,
                ));
            app.manage(remaining_store);
            #[cfg(target_os = "macos")]
            {
                install_help_menu(app)?;
                install_main_window_close_guard(app);
                install_popover_window(app)?;
                install_tray_icon(app)?;
            }
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
            commands::open_main_window,
            commands::open_main_window_settings,
            commands::open_main_window_help,
            commands::hide_popover,
            commands::set_popover_height,
            commands::app_update::download_app_update,
            commands::app_update::restart_app,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Tauri application");

    app.run(|_app_handle, _event| {
        // Dock icon "reopen" gesture (clicking the Dock icon, or the standard
        // app-reactivation event, when the app has no visible windows or is
        // simply not frontmost) — opens/activates the Main Window only, never
        // the Popover. See docs/desktop/mac-popover.md#entry-points.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = _event {
            if let Some(main) = _app_handle.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = main.show();
                let _ = main.set_focus();
            }
        }
    });
}

/// Builds the macOS menu bar. Starts from the app/Edit/Window/Help submenus
/// Tauri would generate by default, but drops File and View (unused by this
/// app) and replaces the app-menu Services item with a Settings item that
/// opens the in-app settings panel.
#[cfg(target_os = "macos")]
fn install_help_menu(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{
        Menu, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID,
    };

    let handle = app.handle();
    let pkg_info = handle.package_info();
    let about_metadata = tauri::menu::AboutMetadata {
        name: Some(pkg_info.name.clone()),
        version: Some(pkg_info.version.to_string()),
        ..Default::default()
    };

    let settings_item = MenuItem::with_id(
        handle,
        MENU_ID_OPEN_SETTINGS,
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

    let menu = Menu::with_items(handle, &[&app_menu, &edit_menu, &window_menu, &help_menu])?;

    app.set_menu(menu)?;
    Ok(())
}

/// Intercepts the Main Window's own close request (red traffic light /
/// Cmd+W) and hides it instead of letting it destroy the window, so the
/// application keeps running with the tray icon and Popover available after
/// the Main Window is "closed" — see docs/desktop/mac-popover.md#closing.
///
/// Quit (Cmd+Q, or the native Quit item installed in `install_help_menu` via
/// `PredefinedMenuItem::quit`) is unaffected by this: it terminates the app
/// directly through the OS's native menu action rather than going through a
/// window's close-request path, so it bypasses this guard entirely.
#[cfg(target_os = "macos")]
fn install_main_window_close_guard(app: &tauri::App) {
    let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };

    let window_to_hide = main_window.clone();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window_to_hide.hide();
        }
    });
}

/// Creates the macOS menu bar Popover window, pointed at popover.html — the
/// standalone Popover frontend surface built in the prior, frontend-only
/// phase (frontend/popover.html / frontend/modules/popover.js). See
/// docs/desktop/mac-popover.md#static-layout.
///
/// Hidden at startup: per Launch Behavior in the doc, the Popover never
/// opens automatically on any launch path (first launch, normal manual
/// launch) — it only ever opens via an explicit tray icon click, wired up in
/// `install_tray_icon` below. No decorations and not resizable because the
/// Popover "is not an OS window" (docs/desktop/mac-popover.md#static-layout,
/// #signing).
///
/// Two native touches make this read closer to a real system popover and
/// behave like one, documented in docs/desktop/mac-popover.md:
///
/// - `.transparent(true)` + `.effects(...)` applies AppKit's `NSVisualEffectView`
///   "popover" vibrancy material behind the webview content, via Tauri's own
///   built-in `WindowEffectsConfig` wiring (`tauri::window::Effect::Popover`,
///   backed by the `window-vibrancy` crate that Tauri already depends on
///   internally for this — see `tauri-2.11.5/src/vibrancy/macos.rs`). This
///   requires the `macos-private-api` Cargo feature (enabled on the `tauri`
///   dependency in Cargo.toml) and the matching `app.macOSPrivateApi: true`
///   flag in tauri.conf.json — see the note in tauri.conf.json and
///   docs/desktop/mac-popover.md for what that opt-in means.
/// - `set_collection_behavior` (below) fixes a real bug: without it, showing
///   this window while the user is on a different macOS Space switches them
///   back to the Space the app's other windows live on.
/// - `.always_on_top(true)` — a menu bar panel has to float above ordinary
///   application windows the way Control Center and the Wi-Fi/sound panels do;
///   without it the Popover is just another window in the normal z-order and
///   can end up behind whatever was frontmost.
/// - `.shadow(true)` — asks for AppKit's own window shadow
///   (`NSWindow.hasShadow`) instead of a CSS drop shadow painted inside the
///   webview, which cannot extend past the window's own bounds. `true` is
///   tao's default, so this is mostly a statement of intent, but it is paired
///   with `invalidate_native_shadow` below, which is not optional: a
///   transparent window's shadow is derived from its rendered alpha and needs
///   to be recomputed once the content (and its rounded corners) is actually
///   on screen.
#[cfg(target_os = "macos")]
fn install_popover_window(app: &tauri::App) -> tauri::Result<()> {
    use tauri::utils::config::WindowEffectsConfig;
    use tauri::window::{Effect, EffectState};

    let popover = tauri::WebviewWindowBuilder::new(
        app,
        POPOVER_WINDOW_LABEL,
        tauri::WebviewUrl::App("popover.html".into()),
    )
    .title("AI Limits")
    .inner_size(POPOVER_WIDTH, POPOVER_DEFAULT_HEIGHT)
    .resizable(false)
    .decorations(false)
    .visible(false)
    .skip_taskbar(true)
    .focused(false)
    .always_on_top(true)
    .shadow(true)
    // Transparent background is required for the vibrancy material applied
    // just below to actually show through — see the frontend CSS notes in
    // docs/desktop/mac-popover.md (popover.html's own background is made
    // translucent for the same reason).
    .transparent(true)
    .effects(WindowEffectsConfig {
        effects: vec![Effect::Popover],
        state: Some(EffectState::Active),
        radius: Some(POPOVER_CORNER_RADIUS),
        color: None,
    })
    .build()?;

    set_collection_behavior(&popover);

    // Dismiss-on-outside-click: hide (never close/destroy) the Popover when
    // it loses focus — standard macOS popover behavior, per
    // docs/desktop/mac-popover.md#closing.
    //
    // Deliberately unconditional. Every focus-loss path this app can actually
    // produce is one where hiding is the right answer: clicking another app,
    // clicking the Main Window, opening an external URL in the browser, or
    // starting a CLI login in Terminal. The app presents no native sheets,
    // alerts or file dialogs of its own that could take focus while the user
    // still means to keep the Popover open, so there is nothing to
    // distinguish here — see docs/desktop/mac-popover.md#closing.
    let popover_to_hide = popover.clone();
    popover.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = popover_to_hide.hide();
        }
    });

    Ok(())
}

/// Recomputes the Popover's native window shadow. Called right after each
/// `show()`.
///
/// A transparent `NSWindow`'s shadow is derived from the alpha channel of what
/// it actually rendered, and AppKit caches that shape. When the window's
/// content or size changed while it was hidden (both happen here: the vibrancy
/// material's rounded mask is applied to the webview surface, and
/// `set_popover_height` resizes the window between showings), the cached
/// shadow can be stale or missing entirely — a rectangular shadow behind
/// rounded corners, or none at all. `invalidateShadow` discards the cache so
/// the next display pass recomputes it.
#[cfg(target_os = "macos")]
fn invalidate_native_shadow(popover: &tauri::WebviewWindow) {
    use objc2_app_kit::NSWindow;

    let Ok(ns_window_ptr) = popover.ns_window() else {
        return;
    };
    // SAFETY: same contract as `set_collection_behavior` below — `ns_window()`
    // hands back this window's own live `NSWindow`, which outlives this call.
    let ns_window: &NSWindow = unsafe { &*ns_window_ptr.cast() };

    ns_window.invalidateShadow();
}

/// Sets the Popover's native `NSWindow.collectionBehavior` so it appears on
/// whichever macOS Space is currently active, instead of switching the user
/// back to the Space the app's other windows live on — the default behavior
/// for any ordinary `NSWindow` when it's shown/focused via
/// `orderFront`/`makeKeyAndOrderFront`, which is what Tauri's `show()` /
/// `set_focus()` boil down to. This is the same fix menu-bar-popover-style
/// utility apps (Bartender, itsycal, etc.) apply to their own status windows.
///
/// Flags used, via `objc2-app-kit`'s `NSWindowCollectionBehavior` bitflags
/// (the exact objc2/objc2-app-kit versions Tauri's own macOS backend already
/// resolves — see Cargo.lock/Cargo.toml):
///
/// - `CanJoinAllSpaces` — the essential fix: the window is considered part of
///   every Space simultaneously rather than pinned to one, so ordering it
///   front never triggers a Space switch.
/// - `Stationary` — the window doesn't get carried along if the user later
///   switches Spaces with it visible (e.g. via Exposé/Mission Control
///   dragging), consistent with "this is a fixed menu-bar accessory, not a
///   document window that travels with the user's workspace."
/// - `IgnoresCycle` — excludes the Popover from Cmd+`~`/Cmd+Tab-style window
///   cycling and the Window menu's window list, matching `skip_taskbar(true)`
///   already set above and the fact that this is a transient popover, not a
///   window a user would ever want to cycle to directly.
/// - `FullScreenAuxiliary` — lets the Popover appear as an auxiliary window
///   over another app's fullscreen Space, the same way the menu bar itself
///   and other menu-bar-utility popovers remain reachable while some other
///   app is fullscreen.
///
/// Deliberately not used: `Transient`/`Managed` (mutually exclusive with
/// `Stationary`, and not appropriate — this window is explicitly *not*
/// transient/ephemeral to a single Space) and `CanJoinAllApplications` (this
/// window belongs to this app only, never meant to be reparented visually
/// under another app's icon in App Exposé).
///
/// Verification note: this could not be exercised against a real multi-Space
/// macOS session in this sandbox (no display). It's verified instead against
/// `objc2-app-kit` 0.3.2's generated `NSWindow` bindings
/// (`NSWindow::setCollectionBehavior`/`NSWindowCollectionBehavior` in
/// `objc2-app-kit-0.3.2/src/generated/NSWindow.rs`) and Tauri 2.11.5's own
/// `WebviewWindow::ns_window()` implementation/doc example
/// (`tauri-2.11.5/src/window/mod.rs`, `tauri-2.11.5/src/webview/webview_window.rs`),
/// and compiles cleanly against this project's resolved dependency graph.
#[cfg(target_os = "macos")]
fn set_collection_behavior(popover: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    let Ok(ns_window_ptr) = popover.ns_window() else {
        return;
    };
    // SAFETY: `ns_window()` returns the popover's own live NSWindow pointer,
    // valid for the lifetime of the window (which outlives this call, since
    // the window has just been built and is owned by the app). Cast mirrors
    // the pattern Tauri's own docs use for the same handle (see
    // `WebviewWindow::with_webview`'s doc example in webview_window.rs).
    let ns_window: &NSWindow = unsafe { &*ns_window_ptr.cast() };

    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
}

/// Creates the macOS menu bar tray icon. Left-clicking it toggles the
/// Popover window — shows and positions it near the icon if hidden, hides it
/// if already visible — and never touches the Main Window, per
/// docs/desktop/mac-popover.md#entry-points. Right-clicking opens the small
/// context menu built by `build_tray_menu` instead, the interaction any
/// menu bar app is expected to offer.
#[cfg(target_os = "macos")]
fn install_tray_icon(app: &tauri::App) -> tauri::Result<()> {
    use tauri::image::Image;
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    // Purpose-built macOS menu bar template image: monochrome, alpha-only,
    // 18pt rendered at @2x (36×36 px). `icon_as_template(true)` sets
    // `NSImage.isTemplate`, which is what makes AppKit tint it for the current
    // menu bar appearance (light/dark, and inverted while the icon is
    // highlighted) instead of blitting our pixels verbatim. Generated from the
    // app's master artwork by scripts/generate-desktop-icons.sh.
    //
    // Embedded in the binary rather than read from disk at runtime: it ships
    // inside the same `.app` either way, and this removes any resource-path
    // resolution and any chance of a missing-file failure at launch.
    // 36×36 is the only size worth shipping — the tray API takes exactly one
    // image and rescales it to an 18pt height, and macOS has no @3x displays.
    let icon = Image::from_bytes(include_bytes!("../icons/icon-tray.png"))
        .expect("icons/icon-tray.png must be a valid PNG");

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .tooltip("AI Limits")
        .menu(&build_tray_menu(app.handle())?)
        // Left click is the Popover toggle handled below; the menu is for the
        // right click only.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            else {
                return;
            };

            let app = tray.app_handle();
            let Some(popover) = app.get_webview_window(POPOVER_WINDOW_LABEL) else {
                return;
            };

            // Toggle: a second click while the Popover is visible hides it.
            if popover.is_visible().unwrap_or(false) {
                let _ = popover.hide();
                return;
            }

            position_popover_near_tray(&popover, &rect);
            let _ = popover.show();
            let _ = popover.set_focus();
            invalidate_native_shadow(&popover);
        })
        .build(app)?;

    Ok(())
}

/// Right-click menu for the tray icon. Every item routes through the same
/// ids `handle_menu_event` already dispatches for the native app menu, so
/// there is exactly one implementation of "open the Main Window" and "open
/// Settings"; Quit is the standard predefined item, identical to the app
/// menu's.
#[cfg(target_os = "macos")]
fn build_tray_menu(handle: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    Menu::with_items(
        handle,
        &[
            &MenuItem::with_id(
                handle,
                MENU_ID_OPEN_MAIN_WINDOW,
                "Open AI Limits",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                handle,
                MENU_ID_OPEN_SETTINGS,
                "Settings…",
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::quit(handle, None)?,
        ],
    )
}

/// Anchors the Popover under the clicked tray icon: horizontally centered on
/// the icon, a few points below the menu bar, kept fully on the display the
/// icon was clicked on.
///
/// Coordinate handling, which is the fiddly part:
///
/// - `TrayIconEvent::Click`'s `rect` arrives already in *physical* pixels,
///   scaled by the backing scale factor of the display the status item is on
///   (see `get_tray_rect` in tray-icon's macOS backend). Its origin is
///   top-left of the global AppKit screen space, y growing downwards.
/// - `Monitor::position()`/`size()` are likewise "physical", i.e. that
///   monitor's own logical AppKit bounds multiplied by its own scale factor.
/// - `set_position(Logical(..))` is the one unambiguous target: tao converts
///   a logical position straight into a global top-left-origin AppKit point.
///   A *physical* position would instead be divided by whatever scale factor
///   the window happens to sit on *before* the move, which is exactly wrong
///   when the move is to a differently-scaled display.
///
/// So everything is converted to logical points first, and the display is
/// found by testing each monitor's own scale factor: dividing the icon rect by
/// scale factor `s` only lands inside the logical bounds of the monitor whose
/// scale factor really is `s`. `primary_monitor()` — what this used to use —
/// is simply the wrong display whenever the menu bar was clicked on a
/// secondary one.
#[cfg(target_os = "macos")]
fn position_popover_near_tray(popover: &tauri::WebviewWindow, icon_rect: &tauri::Rect) {
    use tauri::{LogicalPosition, LogicalSize};

    let Some(monitor) = monitor_for_tray_icon(popover, icon_rect) else {
        return;
    };

    let scale_factor = monitor.scale_factor();
    let icon_position: LogicalPosition<f64> = icon_rect.position.to_logical(scale_factor);
    let icon_size: LogicalSize<f64> = icon_rect.size.to_logical(scale_factor);
    let screen_position: LogicalPosition<f64> = monitor.position().to_logical(scale_factor);
    let screen_size: LogicalSize<f64> = monitor.size().to_logical(scale_factor);

    let popover_size = popover_logical_size(popover);

    let left = screen_position.x + POPOVER_SCREEN_MARGIN;
    let right = screen_position.x + screen_size.width - popover_size.width - POPOVER_SCREEN_MARGIN;
    let x = (icon_position.x + icon_size.width / 2.0 - popover_size.width / 2.0)
        .clamp(left, right.max(left));

    let top = icon_position.y + icon_size.height + POPOVER_MENU_BAR_GAP;
    let bottom =
        screen_position.y + screen_size.height - popover_size.height - POPOVER_SCREEN_MARGIN;
    let y = top.min(bottom.max(screen_position.y));

    let _ = popover.set_position(tauri::Position::Logical(LogicalPosition::new(x, y)));
}

/// Finds the display the tray icon was clicked on. See the coordinate notes on
/// `position_popover_near_tray` for why the search is "try each monitor's own
/// scale factor" rather than a direct lookup. Falls back to the primary
/// monitor if no monitor claims the icon (should not happen, but the icon
/// rect comes from another crate's geometry maths).
#[cfg(target_os = "macos")]
fn monitor_for_tray_icon(
    popover: &tauri::WebviewWindow,
    icon_rect: &tauri::Rect,
) -> Option<tauri::Monitor> {
    use tauri::{LogicalPosition, LogicalSize};

    let monitors = popover.available_monitors().unwrap_or_default();

    let containing = monitors.into_iter().find(|monitor| {
        let scale_factor = monitor.scale_factor();
        let icon_position: LogicalPosition<f64> = icon_rect.position.to_logical(scale_factor);
        let icon_size: LogicalSize<f64> = icon_rect.size.to_logical(scale_factor);
        let screen_position: LogicalPosition<f64> = monitor.position().to_logical(scale_factor);
        let screen_size: LogicalSize<f64> = monitor.size().to_logical(scale_factor);

        let center_x = icon_position.x + icon_size.width / 2.0;
        let center_y = icon_position.y + icon_size.height / 2.0;

        center_x >= screen_position.x
            && center_x <= screen_position.x + screen_size.width
            && center_y >= screen_position.y
            && center_y <= screen_position.y + screen_size.height
    });

    containing.or_else(|| popover.primary_monitor().ok().flatten())
}

/// The Popover's current outer size in logical points. `outer_size()` is
/// physical and relative to the display the window currently sits on, so it is
/// divided by that same window's scale factor — the two always agree.
#[cfg(target_os = "macos")]
fn popover_logical_size(popover: &tauri::WebviewWindow) -> tauri::LogicalSize<f64> {
    let fallback = tauri::LogicalSize::new(POPOVER_WIDTH, POPOVER_DEFAULT_HEIGHT);

    let (Ok(size), Ok(scale_factor)) = (popover.outer_size(), popover.scale_factor()) else {
        return fallback;
    };

    size.to_logical(scale_factor)
}

/// Routes native menu selections — from the app menu and from the tray icon's
/// right-click menu alike — to the commands that own each action. The
/// commands are the same ones the Popover's own toolbar buttons invoke, so
/// there is one implementation per action rather than one per entry point.
fn handle_menu_event(app: &tauri::AppHandle, menu_id: &str) {
    if menu_id == MENU_ID_OPEN_SETTINGS {
        let _ = commands::open_main_window_settings(app.clone());
        return;
    }

    #[cfg(target_os = "macos")]
    if menu_id == MENU_ID_OPEN_MAIN_WINDOW {
        let _ = commands::open_main_window(app.clone());
        return;
    }

    if let Some(chapter) = menu_id.strip_prefix("help:") {
        let _ = commands::open_main_window_help(app.clone(), Some(chapter.to_string()));
    }
}
