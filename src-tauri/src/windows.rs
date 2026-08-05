//! Window identities and geometry shared by `main.rs` (which builds the
//! windows) and `commands` (which is invoked from the frontend and has to
//! find them again). Single source of truth: these used to be duplicated in
//! both modules with a "keep both in sync" comment.

/// Label of the always-alive Main Window (implicit default from
/// tauri.conf.json's single unnamed window entry).
pub const MAIN_WINDOW_LABEL: &str = "main";

/// Label of the macOS menu bar Popover window (see
/// docs/desktop/mac-popover.md). Not `cfg`-gated: looking the window up on a
/// platform where it is never created simply yields `None`, which every
/// caller already treats as "nothing to do".
pub const POPOVER_WINDOW_LABEL: &str = "popover";

/// Popover width in logical pixels. Matches the width the Popover frontend
/// lays out against (frontend/styles/popover.css).
pub const POPOVER_WIDTH: f64 = 348.0;

/// Height bounds, in logical pixels, for the content-driven Popover height
/// reported by the frontend through the `set_popover_height` command (see its
/// doc comment for the contract). `POPOVER_MAX_HEIGHT` is deliberately kept
/// small enough that a Popover of that height still fits below the menu bar
/// on the shortest display Apple ships (1280×800): 800 − 37pt notch-height
/// menu bar − gap − bottom margin still leaves more than this.
pub const POPOVER_MIN_HEIGHT: f64 = 180.0;
pub const POPOVER_MAX_HEIGHT: f64 = 620.0;

/// Height the Popover window is created with, before the frontend has
/// reported its measured content height.
#[cfg(target_os = "macos")]
pub const POPOVER_DEFAULT_HEIGHT: f64 = 480.0;

/// Corner radius of the native window (its vibrancy material's mask), in
/// logical pixels. The Popover frontend rounds its own content to the same
/// value; keep the two in sync.
#[cfg(target_os = "macos")]
pub const POPOVER_CORNER_RADIUS: f64 = 11.0;

/// Vertical gap between the bottom of the menu bar (i.e. the bottom of the
/// clicked tray icon) and the top of the Popover, in logical pixels — system
/// menu bar panels leave a similar few-point gap rather than butting against
/// the menu bar.
#[cfg(target_os = "macos")]
pub const POPOVER_MENU_BAR_GAP: f64 = 6.0;

/// Minimum distance the Popover keeps from the left/right/bottom edges of the
/// display it is shown on, in logical pixels.
#[cfg(target_os = "macos")]
pub const POPOVER_SCREEN_MARGIN: f64 = 8.0;
