import { APP_UPDATE_CHECK_INTERVAL_MS } from "./constants.js";
import { isAutoUpdateEnabled } from "./settings.js";

let updateBanner = null;
let updateBannerText = null;
let updateRestartButton = null;
let updateDismissButton = null;
let checkTimer = null;
let checkInProgress = false;
let stagedVersion = null;

/// Wires the update banner and starts the automatic check schedule. Checks run
/// once at startup and then every APP_UPDATE_CHECK_INTERVAL_MS while the app
/// stays open, but only while the user keeps automatic updates enabled.
export function initAppUpdates({ banner, bannerText, restartButton, dismissButton }) {
  updateBanner = banner;
  updateBannerText = bannerText;
  updateRestartButton = restartButton;
  updateDismissButton = dismissButton;

  updateRestartButton.addEventListener("click", () => {
    restartForUpdate();
  });

  updateDismissButton.addEventListener("click", () => {
    updateBanner.hidden = true;
  });

  syncAppUpdateSchedule();
}

/// Starts or stops the check schedule after the user toggles the setting.
export function syncAppUpdateSchedule() {
  if (!isAutoUpdateEnabled()) {
    stopSchedule();
    return;
  }

  if (checkTimer !== null) {
    return;
  }

  checkTimer = setInterval(checkForUpdate, APP_UPDATE_CHECK_INTERVAL_MS);
  checkForUpdate();
}

function stopSchedule() {
  if (checkTimer === null) {
    return;
  }

  clearInterval(checkTimer);
  checkTimer = null;
}

/// Downloads and installs an available update without interrupting the user.
/// The new version only takes effect once the app restarts, so a ready update
/// is announced in the banner instead of being applied immediately.
async function checkForUpdate() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke || checkInProgress || stagedVersion !== null || !isAutoUpdateEnabled()) {
    return;
  }

  checkInProgress = true;
  try {
    const staged = await invoke("download_app_update");
    if (staged) {
      showStagedUpdate(staged.version);
    }
  } catch {
    // An unreachable manifest or a failed download is not actionable for the
    // user: the next scheduled check retries it.
  } finally {
    checkInProgress = false;
  }
}

function showStagedUpdate(version) {
  stagedVersion = version;
  updateBannerText.textContent = `Version ${version} is ready. Restart to apply it.`;
  updateBanner.hidden = false;
}

async function restartForUpdate() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return;
  }

  try {
    await invoke("restart_app");
  } catch {
    // The app exits on success, so reaching this point means the restart failed
    // and the staged update stays pending until the user quits on their own.
  }
}
