export const updateFrequencyOptions = [
  "Manual only",
  "1 min",
  "5 min",
  "10 min",
  "30 min",
  "1 hour",
];

export const DEFAULT_UPDATE_FREQUENCY = "5 min";
export const SETTINGS_STORAGE_KEY = "ai-limits-settings";
export const PROVIDER_INTERVALS_STORAGE_KEY = "ai-limits-provider-intervals";
export const THEME_STORAGE_KEY = "ai-limits-theme";
export const PROVIDER_IDS = ["codex", "claude", "cursor"];

// Tauri app-wide event name emitted (via window.__TAURI__.event.emit) after a
// settings change is saved to localStorage, so other open windows in the
// same app process — currently, the macOS Menu Bar Popover — can react
// without polling or a `storage` event (which does not fire in the same
// window that wrote the value, and is unreliable cross-window timing-wise
// for this app's needs). See handleSettingsChange/handleDisplaySettingsChange
// in settings.js (emitters) and popover.js (listener).
//
// Payload shape: { kind: "visibility" | "display" }. `kind` documents which
// handler fired for readers that care to distinguish, but listeners are free
// to treat the two identically (re-derive their own state from
// localStorage/settings.js either way) — that's what popover.js does, since
// both a provider-visibility change (tab list) and a display-toggle change
// (card sections) are cheap, idempotent, and safe to redo unconditionally.
export const SETTINGS_CHANGED_EVENT = "settings-changed";

// Tauri app-wide event emitted after a *manual* theme change is saved (the
// "Dark theme" toggle in the Main Window's settings), so other open windows
// re-read it — same reason SETTINGS_CHANGED_EVENT exists: each webview window
// keeps its own module-scoped copy of theme.js's `appTheme`, so localStorage
// alone does not update an already-open window. Kept separate from
// SETTINGS_CHANGED_EVENT because the theme lives in its own storage key and
// its own module, and reacting to it means re-applying `data-theme` rather
// than re-rendering anything. System (non-manual) theme changes need no event:
// every window observes the `prefers-color-scheme` media query itself.
// See setManualTheme in theme.js (emitter) and popover.js (listener).
export const THEME_CHANGED_EVENT = "theme-changed";

// Tauri app-wide event emitted by the backend (not by any frontend module)
// after every successful actual provider collection, carrying the same
// `ProviderLimits` shape a direct `get_single_provider_limits` response
// does. Both Main Window and Popover listen so a collection started by
// either surface updates the other's already-mounted card without a second
// collection — see docs/desktop/ui/frontend-state.md and
// docs/desktop/mac-popover.md#cross-window-sync. Forwarded to the Popover by
// `popover_panel::install_event_forwarding` (src-tauri/src/popover_panel.rs)
// the same way SETTINGS_CHANGED_EVENT/THEME_CHANGED_EVENT are.
export const PROVIDER_UPDATED_EVENT = "provider-updated";

export const EXTERNAL_LINKS = {
  claude: "https://code.claude.com/docs/en/setup",
  codex: "https://developers.openai.com/codex/cli",
  github: "https://github.com/md2it/ai-limits",
  license: "https://github.com/md2it/ai-limits/blob/main/LICENSE",
};

export const DEFAULT_APP_SETTINGS = {
  notifications: true,
  autoUpdate: true,
  cursor: true,
  cloud: true,
  codex: true,
  showLimits: true,
  showPlan: true,
  showSource: false,
  showUpdateTime: false,
};

/// How long the app waits between automatic update checks while it stays open.
/// A check also runs once at startup.
export const APP_UPDATE_CHECK_INTERVAL_MS = 12 * 60 * 60 * 1000;
