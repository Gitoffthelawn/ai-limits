import {
  DEFAULT_UPDATE_FREQUENCY,
  PROVIDER_IDS,
  PROVIDER_INTERVALS_STORAGE_KEY,
  PROVIDER_STATUS_HIDE_MS,
  THEME_ACCENTS,
  updateFrequencyOptions,
} from "./constants.js";
import {
  colorForRemaining,
  escapeHtml,
  formatDecimal,
  formatSourceIdLine,
  formatSourceTimestampLine,
  formatTimestampForDisplay,
} from "./provider-formatters.js";
import {
  attachSourcePriorityControls,
  buildSourcePriorityControlHtml,
  isProviderEnabled,
  settingsToQuery,
  syncSourcePriorityControls,
} from "./settings.js";
import { openHelp } from "./help.js";
import { isScreenshotShowcase, SHOWCASE_PROVIDERS } from "./showcase.js";

let providerList = null;
let statusLine = null;

const providerRefreshIntervals = new Map();
const providerRefreshTimers = new Map();
const providerRefreshInFlight = new Set();
const providerStatusHideTimers = new Map();
const providerDataCache = new Map();

export function initProviders(elements) {
  providerList = elements.providerList;
  statusLine = elements.statusLine;
}

function normalizeUpdateFrequency(frequency) {
  if (typeof frequency !== "string") {
    return DEFAULT_UPDATE_FREQUENCY;
  }

  return updateFrequencyOptions.includes(frequency)
    ? frequency
    : DEFAULT_UPDATE_FREQUENCY;
}

function loadProviderIntervalsFromStorage() {
  try {
    const stored = localStorage.getItem(PROVIDER_INTERVALS_STORAGE_KEY);
    if (!stored) {
      return {};
    }

    const parsed = JSON.parse(stored);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {};
    }

    const intervals = {};
    for (const providerId of PROVIDER_IDS) {
      intervals[providerId] = normalizeUpdateFrequency(parsed[providerId]);
    }
    return intervals;
  } catch {
    return {};
  }
}

export function initProviderIntervals() {
  const stored = loadProviderIntervalsFromStorage();
  for (const providerId of PROVIDER_IDS) {
    providerRefreshIntervals.set(
      providerId,
      stored[providerId] ?? DEFAULT_UPDATE_FREQUENCY,
    );
  }
}

function saveProviderIntervals() {
  const payload = {};
  for (const providerId of PROVIDER_IDS) {
    payload[providerId] = getProviderInterval(providerId);
  }
  localStorage.setItem(PROVIDER_INTERVALS_STORAGE_KEY, JSON.stringify(payload));
}

function frequencyToMs(frequency) {
  switch (frequency) {
    case "1 min":
      return 60_000;
    case "5 min":
      return 300_000;
    case "10 min":
      return 600_000;
    case "30 min":
      return 1_800_000;
    case "1 hour":
      return 3_600_000;
    default:
      return null;
  }
}

function getProviderInterval(providerId) {
  return providerRefreshIntervals.get(providerId) ?? DEFAULT_UPDATE_FREQUENCY;
}

function ensureProviderInterval(providerId, fallbackFrequency) {
  if (!providerRefreshIntervals.has(providerId)) {
    providerRefreshIntervals.set(providerId, normalizeUpdateFrequency(fallbackFrequency));
  }
}

function stopProviderRefreshTimer(providerId) {
  const timerId = providerRefreshTimers.get(providerId);
  if (timerId == null) {
    return;
  }

  clearInterval(timerId);
  providerRefreshTimers.delete(providerId);
}

function restartProviderRefreshTimer(providerId) {
  stopProviderRefreshTimer(providerId);

  if (!isProviderEnabled(providerId)) {
    return;
  }

  const intervalMs = frequencyToMs(getProviderInterval(providerId));
  if (intervalMs == null) {
    return;
  }

  const timerId = setInterval(() => {
    refreshSingleProvider(providerId);
  }, intervalMs);
  providerRefreshTimers.set(providerId, timerId);
}

function setProviderInterval(providerId, frequency) {
  if (!updateFrequencyOptions.includes(frequency)) {
    return;
  }

  providerRefreshIntervals.set(providerId, frequency);
  saveProviderIntervals();
  restartProviderRefreshTimer(providerId);
}

function providerLabel(providerId) {
  return providerId.charAt(0).toUpperCase() + providerId.slice(1);
}

function createEmptyProvider(providerId) {
  return {
    id: providerId,
    label: providerLabel(providerId),
    limits: [],
    availableLimitResets: null,
    sourceId: null,
    dataTimestamp: null,
    selectedUpdateFrequency: getProviderInterval(providerId),
    errorMessage: null,
    noFreshData: false,
    authorizationRequired: null,
    pending: true,
  };
}

function formatCreditsLine(provider) {
  if (provider.creditsRemaining == null) {
    return "";
  }

  return `Available credits: ${formatDecimal(provider.creditsRemaining)}`;
}

function buildLimitResetsHtml(provider) {
  if (Number(provider.availableLimitResets) <= 0) {
    return "";
  }

  return `
    <p class="credits-info">Available resets: ${escapeHtml(provider.availableLimitResets)}</p>
  `;
}

function cliAuthorizationCopy(providerKey) {
  if (providerKey === "claude") {
    return {
      message: "You\u2019re not signed in to Claude CLI.",
      signInLabel: "Sign in to Claude",
      loginCommand: "claude login",
    };
  }

  return {
    message: "You\u2019re not signed in to Codex CLI.",
    signInLabel: "Sign in to Codex",
    loginCommand: "codex login",
  };
}

function buildCliAuthorizationHtml(providerKey) {
  const copy = cliAuthorizationCopy(providerKey);
  return `
    <div class="cli-authorization">
      <p class="provider-message">${escapeHtml(copy.message)}</p>
      <button type="button" class="provider-link provider-link--external" data-provider-cli-login="${escapeHtml(providerKey)}">
        ${escapeHtml(copy.signInLabel)}
      </button>
      <p class="cli-authorization-manual">Or run manually: <code>${escapeHtml(copy.loginCommand)}</code></p>
    </div>
  `;
}

function buildNoFreshDataHtml() {
  return `
    <div class="no-fresh-data">
      <p>No fresh limits' data. Try another source mode:</p>
      <div class="segmented-control" data-source-priority-control role="group" aria-label="Source priority">
        ${buildSourcePriorityControlHtml()}
      </div>
      <button type="button" class="provider-link" data-open-source-priority>
        More details
      </button>
    </div>
  `;
}

function buildLimitRowsHtml(provider) {
  if (!provider.limits.length) {
    if (provider.pending) {
      return "";
    }

    if (provider.availableLimitResets != null) {
      return "";
    }

    if (provider.authorizationRequired) {
      return buildCliAuthorizationHtml(provider.authorizationRequired);
    }

    if (provider.noFreshData) {
      return buildNoFreshDataHtml();
    }

    const message = escapeHtml(provider.errorMessage || "No usable limit records from this source");
    const details = provider.errorMessage === "Local provider data is outdated"
      ? `<button type="button" class="provider-link" data-open-data-errors>More details</button>`
      : "";
    return `<div><p class="provider-message">${message}</p>${details}</div>`;
  }

  return provider.limits
    .map((limit) => {
      const remaining = Number(limit.remainingPercentage) || 0;
      const percent = remaining.toFixed(1);
      const width = Math.max(0, Math.min(100, remaining));
      const fillColor = colorForRemaining(remaining, THEME_ACCENTS);
      const formattedResetTime = formatTimestampForDisplay(limit.resetTime);
      const resetText = formattedResetTime ? `reset ${escapeHtml(formattedResetTime)}` : "";

      return `
        <div class="limit-row">
          <div class="limit-top">${escapeHtml(limit.label)} | ${percent}% left</div>
          <div class="meter" aria-label="${escapeHtml(provider.label)} ${escapeHtml(limit.label)} ${percent}% left">
            <span style="width: ${width}%; background: ${fillColor}"></span>
          </div>
          ${resetText ? `<div class="limit-reset">${resetText}</div>` : ""}
        </div>
      `;
    })
    .join("");
}

function renderProvider(provider) {
  const block = document.createElement("article");
  block.className = "provider-block";
  block.dataset.providerId = provider.id;

  const frequencyOptions = updateFrequencyOptions
    .map((option) => {
      const selected = option === getProviderInterval(provider.id) ? "selected" : "";
      return `<option ${selected}>${option}</option>`;
    })
    .join("");

  block.innerHTML = `
    <div class="provider-status" hidden aria-live="polite">
      <span class="provider-status-indicator" aria-hidden="true"></span>
      <span class="provider-status-text"></span>
    </div>
    <div class="provider-content">
      <div class="provider-header">
        <h2>${escapeHtml(provider.label)}</h2>
      </div>
      <div class="limits">${buildLimitRowsHtml(provider)}</div>
      <p class="credits-info" ${provider.creditsRemaining == null ? "hidden" : ""}>
        ${escapeHtml(formatCreditsLine(provider))}
      </p>
      <div class="limit-resets-slot">${buildLimitResetsHtml(provider)}</div>
      <p class="source-info">
        <span class="source-id">${escapeHtml(formatSourceIdLine(provider))}</span>
        <span class="source-timestamp">${escapeHtml(formatSourceTimestampLine(provider))}</span>
      </p>
    </div>
    <div class="provider-actions">
      <label class="frequency-row">
        <span>Upd&nbsp;every</span>
        <select aria-label="${escapeHtml(provider.label)} update interval">${frequencyOptions}</select>
      </label>
      <button type="button" class="provider-manual-refresh" data-manual-refresh>
        UPDATE NOW
      </button>
    </div>
  `;

  return block;
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

function updateProviderBlockData(block, provider) {
  block.querySelector(".limits").innerHTML = buildLimitRowsHtml(provider);

  const creditsInfo = block.querySelector(".credits-info");
  const creditsLine = formatCreditsLine(provider);
  if (creditsLine) {
    creditsInfo.hidden = false;
    creditsInfo.textContent = creditsLine;
  } else {
    creditsInfo.hidden = true;
    creditsInfo.textContent = "";
  }

  block.querySelector(".limit-resets-slot").innerHTML = buildLimitResetsHtml(provider);
  block.querySelector(".source-id").textContent = formatSourceIdLine(provider);
  block.querySelector(".source-timestamp").textContent = formatSourceTimestampLine(provider);
  attachSourcePriorityBlockHandlers(block);
  attachCliAuthorizationHandlers(block);
}

function isProviderRefreshSuccess(provider) {
  return !provider.errorMessage && !provider.authorizationRequired;
}

function attachProviderBlockHandlers(block, providerId) {
  const select = block.querySelector("select");
  select.value = getProviderInterval(providerId);
  select.addEventListener("change", (event) => {
    setProviderInterval(providerId, event.target.value);
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
  const block = renderProvider(provider);
  attachProviderBlockHandlers(block, provider.id);
  restartProviderRefreshTimer(provider.id);
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

    const block = mountProviderBlock(createEmptyProvider(providerId));
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

  if (showLoading) {
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
    }

    if (isProviderRefreshSuccess(provider)) {
      setProviderStatus(providerId, "updated", "Updated");
    } else {
      setProviderStatus(providerId, "failed", "Failed");
    }
  } catch {
    if (isProviderEnabled(providerId)) {
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
        mountProviderBlock(createEmptyProvider(providerId)),
      ),
    );
  }

  for (const providerId of enabledProviders) {
    setProviderStatus(providerId, "loading");
  }

  for (const providerId of enabledProviders) {
    refreshSingleProvider(providerId, { showLoading: false });
  }
}
