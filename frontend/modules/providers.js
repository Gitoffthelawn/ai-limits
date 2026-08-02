import {
  PROVIDER_IDS,
  PROVIDER_STATUS_HIDE_MS,
} from "./constants.js";
import {
  attachSourcePriorityControls,
  isProviderEnabled,
  settingsToQuery,
  syncSourcePriorityControls,
} from "./settings.js";
import { openHelp } from "./help.js";
import { isScreenshotShowcase, SHOWCASE_PROVIDERS } from "./showcase.js";
import { ensureProviderInterval, getProviderInterval, restartProviderRefreshTimer, setProviderInterval, stopProviderRefreshTimer } from "./provider-refresh-intervals.js";
import { createEmptyProvider, renderProvider, updateProviderBlockData } from "./provider-rendering.js";

export { initProviderIntervals } from "./provider-refresh-intervals.js";

let providerList = null;
let statusLine = null;

const providerRefreshInFlight = new Set();
const providerStatusHideTimers = new Map();
const providerDataCache = new Map();

export function initProviders(elements) {
  providerList = elements.providerList;
  statusLine = elements.statusLine;
}

function getProviderBlock(providerId) {
  return providerList.querySelector(`[data-provider-id="${providerId}"]`);
}

function clearProviderStatusHideTimer(providerId) {
  const timerId = providerStatusHideTimers.get(providerId);
  if (timerId == null) {
    return;
  }

  clearTimeout(timerId);
  providerStatusHideTimers.delete(providerId);
}

function hideProviderStatus(providerId) {
  const block = getProviderBlock(providerId);
  if (!block) {
    return;
  }

  const status = block.querySelector(".provider-status");
  status.hidden = true;
  status.classList.remove("loading", "updated", "failed");
  status.querySelector(".provider-status-text").textContent = "";
}

function scheduleHideProviderStatus(providerId) {
  clearProviderStatusHideTimer(providerId);

  const timerId = setTimeout(() => {
    providerStatusHideTimers.delete(providerId);
    hideProviderStatus(providerId);
  }, PROVIDER_STATUS_HIDE_MS);
  providerStatusHideTimers.set(providerId, timerId);
}

function setProviderStatus(providerId, statusType, statusText = "") {
  const block = getProviderBlock(providerId);
  if (!block) {
    return;
  }

  clearProviderStatusHideTimer(providerId);

  const status = block.querySelector(".provider-status");
  status.hidden = false;
  status.classList.remove("loading", "updated", "failed");
  status.classList.add(statusType);
  status.querySelector(".provider-status-text").textContent =
    statusText || (statusType === "loading" ? "Updating" : "");

  if (statusType === "updated" || statusType === "failed") {
    scheduleHideProviderStatus(providerId);
  }
}

function isProviderRefreshSuccess(provider) {
  return !provider.errorMessage && !provider.authorizationRequired;
}

function attachProviderBlockHandlers(block, providerId) {
  const select = block.querySelector("select");
  select.value = getProviderInterval(providerId);
  select.addEventListener("change", (event) => {
    setProviderInterval(providerId, event.target.value, refreshSingleProvider);
  });
  block.querySelector("[data-manual-refresh]")?.addEventListener("click", () => {
    refreshSingleProvider(providerId);
  });
  attachSourcePriorityBlockHandlers(block);
  attachCliAuthorizationHandlers(block);
}

function attachSourcePriorityBlockHandlers(block) {
  const control = block.querySelector("[data-source-priority-control]");
  if (control) {
    attachSourcePriorityControls(control);
    syncSourcePriorityControls();
  }

  const detailsButton = block.querySelector("[data-open-source-priority]");
  if (detailsButton) {
    detailsButton.addEventListener("click", () => {
      openHelp("source-priority");
    });
  }

  const dataErrorsButton = block.querySelector("[data-open-data-errors]");
  if (dataErrorsButton) {
    dataErrorsButton.addEventListener("click", () => {
      openHelp("data-errors");
    });
  }
}

function attachCliAuthorizationHandlers(block) {
  const loginButton = block.querySelector("[data-provider-cli-login]");
  if (!loginButton) {
    return;
  }

  loginButton.addEventListener("click", () => {
    startProviderCliLogin(loginButton.dataset.providerCliLogin);
  });
}

async function startProviderCliLogin(provider) {
  if (provider !== "codex" && provider !== "claude") {
    return;
  }

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return;
  }

  try {
    await invoke("start_provider_cli_login", { provider });
  } catch (error) {
    setErrorState(error?.message || String(error) || "Could not start provider sign-in.");
  }
}

function mountProviderBlock(provider) {
  ensureProviderInterval(provider.id, provider.selectedUpdateFrequency);
  const block = renderProvider(provider, getProviderInterval(provider.id));
  attachProviderBlockHandlers(block, provider.id);
  restartProviderRefreshTimer(provider.id, refreshSingleProvider);
  return block;
}

function setErrorState(message) {
  statusLine.hidden = false;
  statusLine.classList.remove("loading");
  statusLine.classList.add("error");
  statusLine.textContent = message;
}

function clearStatusState() {
  statusLine.textContent = "";
  statusLine.hidden = true;
  statusLine.classList.remove("loading", "error");
}

function cacheProviderData(provider) {
  providerDataCache.set(provider.id, { ...provider, pending: false });
}

function hasCachedProviderData(providerId) {
  const cached = providerDataCache.get(providerId);
  return cached != null && !cached.pending;
}

function insertProviderBlockInOrder(block, providerId) {
  const enabledProviders = PROVIDER_IDS.filter(isProviderEnabled);
  const targetIndex = enabledProviders.indexOf(providerId);

  for (let i = targetIndex + 1; i < enabledProviders.length; i += 1) {
    const nextBlock = getProviderBlock(enabledProviders[i]);
    if (nextBlock) {
      providerList.insertBefore(block, nextBlock);
      return;
    }
  }

  providerList.appendChild(block);
}

export function removeDisabledProviderBlocks() {
  for (const providerId of PROVIDER_IDS) {
    if (isProviderEnabled(providerId)) {
      continue;
    }

    stopProviderRefreshTimer(providerId);
    clearProviderStatusHideTimer(providerId);
    getProviderBlock(providerId)?.remove();
  }
}

export function restoreNewlyEnabledProviders(providerIds) {
  if (!providerIds.length) {
    return;
  }

  clearStatusState();

  for (const providerId of providerIds) {
    if (getProviderBlock(providerId)) {
      continue;
    }

    if (hasCachedProviderData(providerId)) {
      const block = mountProviderBlock(providerDataCache.get(providerId));
      insertProviderBlockInOrder(block, providerId);
      continue;
    }

    const block = mountProviderBlock(createEmptyProvider(providerId, getProviderInterval(providerId)));
    insertProviderBlockInOrder(block, providerId);
    refreshSingleProvider(providerId);
  }
}

async function fetchSingleProviderLimits(providerId) {
  if (isScreenshotShowcase) {
    return {
      ...SHOWCASE_PROVIDERS[providerId],
      limits: SHOWCASE_PROVIDERS[providerId].limits.map((limit) => ({ ...limit })),
    };
  }

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    throw new Error("Tauri API is unavailable");
  }

  return invoke("get_single_provider_limits", {
    providerId,
    query: settingsToQuery(),
  });
}

async function refreshSingleProvider(providerId, { showLoading = true } = {}) {
  if (!isProviderEnabled(providerId) || providerRefreshInFlight.has(providerId)) {
    return;
  }

  providerRefreshInFlight.add(providerId);

  if (showLoading && !isScreenshotShowcase) {
    setProviderStatus(providerId, "loading");
  }

  try {
    const provider = await fetchSingleProviderLimits(providerId);
    provider.pending = false;
    cacheProviderData(provider);

    if (!isProviderEnabled(providerId)) {
      return;
    }

    let block = getProviderBlock(providerId);
    if (!block) {
      block = mountProviderBlock(provider);
      insertProviderBlockInOrder(block, providerId);
    } else {
      updateProviderBlockData(block, provider);
      attachSourcePriorityBlockHandlers(block);
      attachCliAuthorizationHandlers(block);
    }

    if (isScreenshotShowcase) {
      return;
    }

    if (isProviderRefreshSuccess(provider)) {
      setProviderStatus(providerId, "updated", "Updated");
    } else {
      setProviderStatus(providerId, "failed", "Failed");
    }
  } catch {
    if (isProviderEnabled(providerId) && !isScreenshotShowcase) {
      setProviderStatus(providerId, "failed", "Failed");
    }
  } finally {
    providerRefreshInFlight.delete(providerId);
  }
}

export function refreshEnabledProviders({ initial = false } = {}) {
  removeDisabledProviderBlocks();

  const enabledProviders = PROVIDER_IDS.filter(isProviderEnabled);
  if (!enabledProviders.length) {
    providerList.replaceChildren();
    clearStatusState();
    setErrorState("Enable at least one provider in settings.");
    return;
  }

  clearStatusState();

  if (initial) {
    providerList.replaceChildren(
      ...enabledProviders.map((providerId) =>
        mountProviderBlock(createEmptyProvider(providerId, getProviderInterval(providerId))),
      ),
    );
  }

  if (!isScreenshotShowcase) {
    for (const providerId of enabledProviders) {
      setProviderStatus(providerId, "loading");
    }
  }

  for (const providerId of enabledProviders) {
    refreshSingleProvider(providerId, { showLoading: false });
  }
}
