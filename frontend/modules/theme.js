import { THEME_CHANGED_EVENT, THEME_STORAGE_KEY } from "./constants.js";

let darkThemeInput = null;
let appTheme = { mode: "system", value: "light" };

export function initTheme(input) {
  darkThemeInput = input;
  appTheme = loadAppTheme();
}

export function getAppTheme() {
  return appTheme;
}

function getSystemTheme() {
  if (typeof window.matchMedia !== "function") {
    return "light";
  }

  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }

  if (window.matchMedia("(prefers-color-scheme: light)").matches) {
    return "light";
  }

  return "light";
}

function loadAppTheme() {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (!stored) {
      return { mode: "system", value: getSystemTheme() };
    }

    const parsed = JSON.parse(stored);
    if (
      parsed &&
      parsed.mode === "manual" &&
      (parsed.value === "dark" || parsed.value === "light")
    ) {
      return { mode: "manual", value: parsed.value };
    }

    return { mode: "system", value: getSystemTheme() };
  } catch {
    return { mode: "system", value: getSystemTheme() };
  }
}

function saveAppTheme() {
  localStorage.setItem(THEME_STORAGE_KEY, JSON.stringify(appTheme));
}

export function applyAppTheme() {
  document.documentElement.dataset.theme = appTheme.value;
  if (darkThemeInput) {
    darkThemeInput.checked = appTheme.value === "dark";
  }
}

export function setManualTheme(isDark) {
  appTheme = { mode: "manual", value: isDark ? "dark" : "light" };
  saveAppTheme();
  applyAppTheme();
  emitThemeChanged();
}

// Notifies other open windows in this app process (currently the Popover) of
// a manual theme change, mirroring settings.js's emitSettingsChanged. Only
// under Tauri: a plain browser context has no cross-window IPC and no other
// window to receive it.
function emitThemeChanged() {
  const emit = window.__TAURI__?.event?.emit;
  if (!emit) {
    return;
  }

  emit(THEME_CHANGED_EVENT, { mode: appTheme.mode, value: appTheme.value }).catch(() => {});
}

// Re-reads the theme from localStorage into this window's own module-scoped
// `appTheme` and applies it. The counterpart of the emit above, for a window
// reacting to THEME_CHANGED_EVENT.
export function reloadAppTheme() {
  appTheme = loadAppTheme();
  applyAppTheme();
}

export function syncSystemTheme() {
  if (appTheme.mode === "manual") {
    return;
  }

  appTheme = { mode: "system", value: getSystemTheme() };
  applyAppTheme();
}
