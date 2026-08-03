import { initTheme, applyAppTheme, setManualTheme, syncSystemTheme } from "./theme.js";
import {
  initSettings,
  syncSettingsInputs,
  setSettingsMenuOpen,
  handleSettingsChange,
  handleDisplaySettingsChange,
} from "./settings.js";
import { initAppUpdates, syncAppUpdateSchedule } from "./app-update.js";
import { initHelp, renderHelpMenu, openHelp, closeHelp, isHelpOpen } from "./help.js";
import { setupScreenshotShowcase } from "./showcase.js";
import {
  initProviders,
  initProviderIntervals,
  refreshEnabledProviders,
  refreshProviderSectionsFromCache,
  removeDisabledProviderBlocks,
  scheduleSectionSlotAlignment,
  restoreNewlyEnabledProviders,
} from "./providers.js";

const providerList = document.querySelector("#provider-list");
const statusLine = document.querySelector("#status-line");
const refreshButton = document.querySelector("#refresh-button");
const settingsButton = document.querySelector("#settings-button");
const settingsDropdown = document.querySelector("#settings-dropdown");
const helpButton = document.querySelector("#help-button");
const helpView = document.querySelector("#help-view");
const homeView = document.querySelector("#home-view");
const helpMenu = document.querySelector("#help-menu");
const helpContent = document.querySelector("#help-content");
const helpBack = document.querySelector("#help-back");
const settingInputs = {
  notifications: document.querySelector("#setting-notifications"),
  cursor: document.querySelector("#setting-cursor"),
  cloud: document.querySelector("#setting-cloud"),
  codex: document.querySelector("#setting-codex"),
  showLimits: document.querySelector("#setting-show-limits"),
  showPlan: document.querySelector("#setting-show-plan"),
  showUsage: document.querySelector("#setting-show-usage"),
  sourcePriority: document.querySelector("#setting-source-priority"),
  sourcePriorityInfo: document.querySelector("#setting-source-priority-info"),
  autoUpdate: document.querySelector("#setting-auto-update"),
  darkTheme: document.querySelector("#setting-dark-theme"),
};

const updateBannerEls = {
  banner: document.querySelector("#update-banner"),
  bannerText: document.querySelector("#update-banner-text"),
  restartButton: document.querySelector("#update-restart"),
  dismissButton: document.querySelector("#update-dismiss"),
};

const menuEls = { settingsDropdown, settingsButton };

function closeSettingsMenu() {
  setSettingsMenuOpen(false, menuEls);
}

initTheme(settingInputs.darkTheme);
initProviders({ providerList, statusLine });
initSettings(settingInputs, {
  onChanged({ newlyEnabled }) {
    removeDisabledProviderBlocks();
    restoreNewlyEnabledProviders(newlyEnabled);
    syncAppUpdateSchedule();
  },
  onDisplayChanged() {
    refreshProviderSectionsFromCache();
  },
});
initAppUpdates(updateBannerEls);
initHelp(
  { helpMenu, helpContent, helpView, homeView },
  { onCloseSettings: closeSettingsMenu },
);

settingsButton.addEventListener("click", (event) => {
  event.stopPropagation();
  setSettingsMenuOpen(settingsDropdown.hidden, menuEls);
});

settingsDropdown.addEventListener("click", (event) => {
  event.stopPropagation();
});

settingInputs.sourcePriorityInfo.addEventListener("click", (event) => {
  event.stopPropagation();
  openHelp("source-priority");
});

helpButton.addEventListener("click", (event) => {
  event.stopPropagation();
  openHelp();
});

helpBack.addEventListener("click", () => {
  closeHelp();
});

// Bridge for the macOS native Help menu (see src-tauri/src/main.rs).
window.__openHelpFromNative = (chapterId) => {
  openHelp(chapterId);
};

// Bridge for the macOS native AI Limits > Settings… menu item (see src-tauri/src/main.rs).
window.__openSettingsFromNative = () => {
  setSettingsMenuOpen(true, menuEls);
};

const displaySettingInputs = [settingInputs.showLimits, settingInputs.showPlan, settingInputs.showUsage];

for (const input of Object.values(settingInputs)) {
  if (input === settingInputs.sourcePriority || input === settingInputs.sourcePriorityInfo) {
    continue;
  }

  if (input === settingInputs.darkTheme) {
    input.addEventListener("change", (event) => {
      setManualTheme(event.target.checked);
    });
    continue;
  }

  if (displaySettingInputs.includes(input)) {
    input.addEventListener("change", handleDisplaySettingsChange);
    continue;
  }

  input.addEventListener("change", handleSettingsChange);
}

if (typeof window.matchMedia === "function") {
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", syncSystemTheme);
  window.matchMedia("(prefers-color-scheme: light)").addEventListener("change", syncSystemTheme);
}

window.addEventListener("resize", scheduleSectionSlotAlignment);

document.addEventListener("click", () => {
  closeSettingsMenu();
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeSettingsMenu();
    if (isHelpOpen()) {
      closeHelp();
    }
  }
});

renderHelpMenu();
setupScreenshotShowcase();
initProviderIntervals();
applyAppTheme();
syncSettingsInputs();
refreshButton.addEventListener("click", () => {
  refreshEnabledProviders();
});
refreshEnabledProviders({ initial: true });
