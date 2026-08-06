export const updateFrequencyOptions = [
  "Manual only",
  "1 min",
  "5 min",
  "10 min",
  "30 min",
  "1 hour",
];

export const DEFAULT_UPDATE_FREQUENCY = "10 min";
export const SETTINGS_STORAGE_KEY = "ai-limits-settings";
export const THEME_STORAGE_KEY = "ai-limits-theme";
export const PROVIDER_IDS = ["codex", "claude", "cursor"];

// Tauri app-wide event name emitted (via window.__TAURI__.event.emit) after a
// settings change is saved to localStorage, so other open windows in the
// same app process — currently, the macOS Menu Bar Popover — can react
// without polling or a `storage` event (which does not fire in the same
// window that wrote the value, and is unreliable cross-window timing-wise
// for this app's needs). See handleSettingsChange/handleDisplaySettingsChange
// /handleUpdateFrequencyChange in settings.js (emitters) and popover.js (listener).
//
// Payload shape: { kind: "visibility" | "display" | "update-frequency" }.
// `kind` documents which handler fired for readers that care to distinguish.
// Listeners may treat them the same (re-derive state from localStorage) or
// branch — popover.js reloads settings and re-renders on any kind, and also
// reapplies refresh schedules so an update-frequency change takes effect
// immediately.
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

// Tauri app-wide event emitted by the backend right as an actual collection
// begins (before the source chain runs), payload `{ id: <provider id> }`.
// `CollectionCoordinator` runs at most one real collection per provider at a
// time, so this fires exactly once per collection no matter which surface
// (or how many) requested it. Both Main Window and Popover listen so a
// refresh started in one surface shows the in-flight animation on the same
// card in the other surface too, without starting a collection of its own.
// Forwarded to the Popover the same way as PROVIDER_UPDATED_EVENT.
export const PROVIDER_REFRESH_STARTED_EVENT = "provider-refresh-started";

// Tauri app-wide event emitted by the backend after a failed actual
// collection, carrying the same `ProviderLimits` shape PROVIDER_UPDATED_EVENT
// does (errorMessage set, limits empty). Lets a surface that did not start
// the collection show the same error state instead of sitting on stale data
// with no explanation. Forwarded to the Popover the same way as
// PROVIDER_UPDATED_EVENT.
export const PROVIDER_REFRESH_FAILED_EVENT = "provider-refresh-failed";

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
  updateFrequency: DEFAULT_UPDATE_FREQUENCY,
};

/// How long the app waits between automatic update checks while it stays open.
/// A check also runs once at startup.
export const APP_UPDATE_CHECK_INTERVAL_MS = 12 * 60 * 60 * 1000;
