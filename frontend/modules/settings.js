import {
  DEFAULT_APP_SETTINGS,
  PROVIDER_IDS,
  SETTINGS_STORAGE_KEY,
  SOURCE_PRIORITY_LABELS,
  SOURCE_PRIORITY_OPTIONS,
} from "./constants.js";
import { getAppTheme } from "./theme.js";

let settingInputs = null;
let appSettings = { ...DEFAULT_APP_SETTINGS };
let onSettingsChanged = null;

export function initSettings(inputs, { onChanged } = {}) {
  settingInputs = inputs;
  onSettingsChanged = onChanged ?? null;
  appSettings = loadAppSettings();
}

export function getAppSettings() {
  return appSettings;
}

function normalizeSourcePriority(value) {
  return SOURCE_PRIORITY_OPTIONS.includes(value) ? value : DEFAULT_APP_SETTINGS.sourcePriority;
}

function migrateSourcePriority(parsed) {
  if (SOURCE_PRIORITY_OPTIONS.includes(parsed.sourcePriority)) {
    return parsed.sourcePriority;
  }

  if (parsed.useCliFallback === true) {
    return "full";
  }

  return DEFAULT_APP_SETTINGS.sourcePriority;
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
      cursor: typeof parsed.cursor === "boolean" ? parsed.cursor : DEFAULT_APP_SETTINGS.cursor,
      cloud: typeof parsed.cloud === "boolean" ? parsed.cloud : DEFAULT_APP_SETTINGS.cloud,
      codex: typeof parsed.codex === "boolean" ? parsed.codex : DEFAULT_APP_SETTINGS.codex,
      sourcePriority: migrateSourcePriority(parsed),
    };
  } catch {
    return { ...DEFAULT_APP_SETTINGS };
  }
}

function saveAppSettings() {
  localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(appSettings));
}

export function buildSourcePriorityControlHtml() {
  return SOURCE_PRIORITY_OPTIONS.map((priority) => {
    const selected = priority === appSettings.sourcePriority;
    return `<button type="button" data-source-priority="${priority}" aria-pressed="${selected}">${SOURCE_PRIORITY_LABELS[priority]}</button>`;
  }).join("");
}

export function syncSourcePriorityControls() {
  for (const control of document.querySelectorAll("[data-source-priority-control]")) {
    for (const button of control.querySelectorAll("[data-source-priority]")) {
      const selected = button.dataset.sourcePriority === appSettings.sourcePriority;
      button.setAttribute("aria-pressed", String(selected));
    }
  }
}

export function setSourcePriority(priority) {
  const normalized = normalizeSourcePriority(priority);
  if (appSettings.sourcePriority === normalized) {
    return;
  }

  appSettings.sourcePriority = normalized;
  saveAppSettings();
  syncSourcePriorityControls();
}

export function attachSourcePriorityControls(container) {
  container.dataset.sourcePriorityControl = "";
  for (const button of container.querySelectorAll("[data-source-priority]")) {
    button.addEventListener("click", () => {
      setSourcePriority(button.dataset.sourcePriority);
    });
  }
}

function renderSettingsSourcePriorityControl() {
  settingInputs.sourcePriority.innerHTML = buildSourcePriorityControlHtml();
  attachSourcePriorityControls(settingInputs.sourcePriority);
}

export function syncSettingsInputs() {
  settingInputs.notifications.checked = appSettings.notifications;
  settingInputs.cursor.checked = appSettings.cursor;
  settingInputs.cloud.checked = appSettings.cloud;
  settingInputs.codex.checked = appSettings.codex;
  settingInputs.darkTheme.checked = getAppTheme().value === "dark";
  renderSettingsSourcePriorityControl();
  syncSourcePriorityControls();
}

export function settingsToQuery() {
  return {
    enabledCodex: appSettings.codex,
    enabledClaude: appSettings.cloud,
    enabledCursor: appSettings.cursor,
    sourcePriority: appSettings.sourcePriority,
    notificationsEnabled: appSettings.notifications,
  };
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
    notifications: settingInputs.notifications.checked,
    cursor: settingInputs.cursor.checked,
    cloud: settingInputs.cloud.checked,
    codex: settingInputs.codex.checked,
    sourcePriority: appSettings.sourcePriority,
  };
  saveAppSettings();

  const newlyEnabled = PROVIDER_IDS.filter(
    (providerId) => isProviderEnabled(providerId) && !previouslyEnabled.includes(providerId),
  );

  syncSourcePriorityControls();
  onSettingsChanged?.({ newlyEnabled });
}
