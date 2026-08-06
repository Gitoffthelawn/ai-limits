import {
  DEFAULT_APP_SETTINGS,
  DEFAULT_UPDATE_FREQUENCY,
  PROVIDER_IDS,
  SETTINGS_CHANGED_EVENT,
  SETTINGS_STORAGE_KEY,
  updateFrequencyOptions,
} from "./constants.js";
import { getAppTheme } from "./theme.js";

let settingInputs = null;
let appSettings = { ...DEFAULT_APP_SETTINGS };
let onSettingsChanged = null;
let onDisplaySettingsChanged = null;
let onUpdateFrequencyChanged = null;

export function initSettings(inputs, { onChanged, onDisplayChanged, onUpdateFrequencyChanged: onFrequencyChanged } = {}) {
  settingInputs = inputs;
  onSettingsChanged = onChanged ?? null;
  onDisplaySettingsChanged = onDisplayChanged ?? null;
  onUpdateFrequencyChanged = onFrequencyChanged ?? null;
  appSettings = loadAppSettings();
}

// Re-reads settings from localStorage into this window's own module-scoped
// `appSettings`. Needed by windows that react to the SETTINGS_CHANGED_EVENT
// emitted below: each Tauri webview window runs its own JS context with its
// own copy of this module, so a settings save in one window (e.g. the Main
// Window) does not update another already-open window's in-memory
// `appSettings` (e.g. the Popover) — only the localStorage value it wrote.
// Call this before re-reading isProviderEnabled/isShowLimitsEnabled/etc. in
// response to the event, otherwise they'd still report the pre-change state.
export function reloadAppSettings() {
  appSettings = loadAppSettings();
}

// Emits SETTINGS_CHANGED_EVENT for other open windows in this app process to
// react to (currently: the Popover — see popover.js). Only under Tauri: in a
// plain-browser context (no window.__TAURI__, e.g. the showcase preview or
// this file loaded directly) there is no cross-window IPC to emit over, and
// no other window to receive it.
function emitSettingsChanged(kind) {
  const emit = window.__TAURI__?.event?.emit;
  if (!emit) {
    return;
  }

  emit(SETTINGS_CHANGED_EVENT, { kind }).catch(() => {});
}

function normalizeStoredUpdateFrequency(frequency) {
  return typeof frequency === "string" && updateFrequencyOptions.includes(frequency)
    ? frequency
    : DEFAULT_UPDATE_FREQUENCY;
}

function loadAppSettings() {
  try {
    const stored = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!stored) {
      return { ...DEFAULT_APP_SETTINGS };
    }

    const parsed = JSON.parse(stored);
    return {
      notifications:
        typeof parsed.notifications === "boolean"
          ? parsed.notifications
          : DEFAULT_APP_SETTINGS.notifications,
      autoUpdate:
        typeof parsed.autoUpdate === "boolean"
          ? parsed.autoUpdate
          : DEFAULT_APP_SETTINGS.autoUpdate,
      cursor: typeof parsed.cursor === "boolean" ? parsed.cursor : DEFAULT_APP_SETTINGS.cursor,
      cloud: typeof parsed.cloud === "boolean" ? parsed.cloud : DEFAULT_APP_SETTINGS.cloud,
      codex: typeof parsed.codex === "boolean" ? parsed.codex : DEFAULT_APP_SETTINGS.codex,
      showLimits: typeof parsed.showLimits === "boolean" ? parsed.showLimits : DEFAULT_APP_SETTINGS.showLimits,
      showPlan: typeof parsed.showPlan === "boolean" ? parsed.showPlan : DEFAULT_APP_SETTINGS.showPlan,
      showSource: typeof parsed.showSource === "boolean" ? parsed.showSource : DEFAULT_APP_SETTINGS.showSource,
      showUpdateTime:
        typeof parsed.showUpdateTime === "boolean"
          ? parsed.showUpdateTime
          : DEFAULT_APP_SETTINGS.showUpdateTime,
      updateFrequency: normalizeStoredUpdateFrequency(parsed.updateFrequency),
    };
  } catch {
    return { ...DEFAULT_APP_SETTINGS };
  }
}

function saveAppSettings() {
  localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(appSettings));
}

export function syncSettingsInputs() {
  settingInputs.notifications.checked = appSettings.notifications;
  settingInputs.autoUpdate.checked = appSettings.autoUpdate;
  settingInputs.cursor.checked = appSettings.cursor;
  settingInputs.cloud.checked = appSettings.cloud;
  settingInputs.codex.checked = appSettings.codex;
  settingInputs.showLimits.checked = appSettings.showLimits;
  settingInputs.showPlan.checked = appSettings.showPlan;
  settingInputs.showSource.checked = appSettings.showSource;
  settingInputs.showUpdateTime.checked = appSettings.showUpdateTime;
  settingInputs.darkTheme.checked = getAppTheme().value === "dark";
  if (settingInputs.updateFrequency) {
    settingInputs.updateFrequency.value = appSettings.updateFrequency;
  }
}

export function isShowLimitsEnabled() {
  return appSettings.showLimits;
}

export function isShowPlanEnabled() {
  return appSettings.showPlan;
}

export function isShowSourceEnabled() {
  return appSettings.showSource;
}

export function isShowUpdateTimeEnabled() {
  return appSettings.showUpdateTime;
}

export function getUpdateFrequency() {
  return appSettings.updateFrequency;
}

export function settingsToQuery() {
  return {
    enabledCodex: appSettings.codex,
    enabledClaude: appSettings.cloud,
    enabledCursor: appSettings.cursor,
    notificationsEnabled: appSettings.notifications,
  };
}

export function isAutoUpdateEnabled() {
  return appSettings.autoUpdate;
}

export function isProviderEnabled(providerId) {
  switch (providerId) {
    case "codex":
      return appSettings.codex;
    case "claude":
      return appSettings.cloud;
    case "cursor":
      return appSettings.cursor;
    default:
      return false;
  }
}

export function handleSettingsChange() {
  const previouslyEnabled = PROVIDER_IDS.filter(isProviderEnabled);

  appSettings = {
    ...appSettings,
    notifications: settingInputs.notifications.checked,
    autoUpdate: settingInputs.autoUpdate.checked,
    cursor: settingInputs.cursor.checked,
    cloud: settingInputs.cloud.checked,
    codex: settingInputs.codex.checked,
  };
  saveAppSettings();

  const newlyEnabled = PROVIDER_IDS.filter(
    (providerId) => isProviderEnabled(providerId) && !previouslyEnabled.includes(providerId),
  );

  onSettingsChanged?.({ newlyEnabled });
  emitSettingsChanged("visibility");
}

// Display toggles (Limits / Plan / Source / Update time) never affect
// what is requested from the backend and never trigger a refresh. They only
// change what the frontend renders from data it already holds, so this
// handler saves the choice and asks the caller to re-render already-mounted
// provider blocks in place, unlike handleSettingsChange above.
export function handleDisplaySettingsChange() {
  appSettings = {
    ...appSettings,
    showLimits: settingInputs.showLimits.checked,
    showPlan: settingInputs.showPlan.checked,
    showSource: settingInputs.showSource.checked,
    showUpdateTime: settingInputs.showUpdateTime.checked,
  };
  saveAppSettings();
  onDisplaySettingsChanged?.();
  emitSettingsChanged("display");
}

// Shared update-frequency setting: saves the choice, reapplies the schedule
// for every enabled provider, and does not itself start a data refresh
// beyond what the recomputed schedule requires (see controls.md).
export function handleUpdateFrequencyChange(frequency) {
  if (!updateFrequencyOptions.includes(frequency) || frequency === appSettings.updateFrequency) {
    return;
  }

  appSettings = {
    ...appSettings,
    updateFrequency: frequency,
  };
  saveAppSettings();
  if (settingInputs.updateFrequency) {
    settingInputs.updateFrequency.value = frequency;
  }
  onUpdateFrequencyChanged?.();
  emitSettingsChanged("update-frequency");
}
