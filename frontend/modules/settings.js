import {
  DEFAULT_APP_SETTINGS,
  PROVIDER_IDS,
  SETTINGS_STORAGE_KEY,
} from "./constants.js";
import { getAppTheme } from "./theme.js";

let settingInputs = null;
let appSettings = { ...DEFAULT_APP_SETTINGS };
let onSettingsChanged = null;
let onDisplaySettingsChanged = null;

export function initSettings(inputs, { onChanged, onDisplayChanged } = {}) {
  settingInputs = inputs;
  onSettingsChanged = onChanged ?? null;
  onDisplaySettingsChanged = onDisplayChanged ?? null;
  appSettings = loadAppSettings();
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

export function setSettingsMenuOpen(isOpen, { settingsDropdown, settingsButton }) {
  settingsDropdown.hidden = !isOpen;
  settingsButton.setAttribute("aria-expanded", String(isOpen));
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
}

// Display toggles (Limits / Subscription / Source / Update time) never affect
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
}
