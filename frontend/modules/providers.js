import {
  PROVIDER_IDS,
} from "./constants.js";
import { isProviderEnabled, settingsToQuery } from "./settings.js";
import { openHelp } from "./help.js";
import { openExternalUrl } from "./links.js";
import { isScreenshotShowcase, SHOWCASE_PROVIDERS } from "./showcase.js";
import { ensureProviderInterval, getProviderInterval, getProviderNextRefreshAt, restartProviderRefreshTimer, setProviderInterval, stopProviderRefreshTimer } from "./provider-refresh-intervals.js";
import { createEmptyProvider, renderProvider, syncFrequencyOptions, updateProviderBlockData, updateProviderUpdateTimeText } from "./provider-rendering.js";

export { initProviderIntervals } from "./provider-refresh-intervals.js";

let providerList = null;
let statusLine = null;

const providerRefreshInFlight = new Set();
const providerDataCache = new Map();
const SECTION_SLOT_KINDS = ["limits", "plan"];
let sectionSlotAlignmentFrame = 0;

export function initProviders(elements) {
  providerList = elements.providerList;
  statusLine = elements.statusLine;
  document.addEventListener("click", closeAllProviderSettingsMenus);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeAllProviderSettingsMenus();
    }
  });
}

function closeAllProviderSettingsMenus() {
  if (!providerList) {
    return;
  }

  for (const dropdown of providerList.querySelectorAll("[data-provider-settings-dropdown]")) {
    dropdown.hidden = true;
  }
  for (const button of providerList.querySelectorAll("[data-provider-settings-button]")) {
    button.setAttribute("aria-expanded", "false");
  }
}

function getProviderBlock(providerId) {
  return providerList.querySelector(`[data-provider-id="${providerId}"]`);
}

function applySectionSlotAlignment() {
  sectionSlotAlignmentFrame = 0;

  if (!providerList) {
    return;
  }

  const sectionsByKind = new Map(SECTION_SLOT_KINDS.map((kind) => [kind, []]));
  const allSections = [];

  // Measuring with the min-height transition still active can read a
  // mid-flight value left over from the previous sync instead of the
  // section's natural content height, producing a stray second jump.
  // Disabling the transition for the reset-and-measure pass keeps the
  // measurement accurate; it's re-enabled next frame so the resulting
  // min-height change still animates normally.
  for (const section of providerList.querySelectorAll(".provider-section[data-section-slot]")) {
    section.style.transition = "none";
    section.style.minHeight = "";
    allSections.push(section);
    const sections = sectionsByKind.get(section.dataset.sectionSlot);
    if (sections) {
      sections.push(section);
    }
  }

  for (const sections of sectionsByKind.values()) {
    if (!sections.length) {
      continue;
    }

    const maxHeight = sections.reduce(
      (height, section) => Math.max(height, section.getBoundingClientRect().height),
      0,
    );

    for (const section of sections) {
      section.style.minHeight = `${maxHeight}px`;
    }
  }

  window.requestAnimationFrame(() => {
    for (const section of allSections) {
      section.style.transition = "";
    }
  });
}

export function scheduleSectionSlotAlignment() {
  if (!providerList || sectionSlotAlignmentFrame) {
    return;
  }

  sectionSlotAlignmentFrame = window.requestAnimationFrame(applySectionSlotAlignment);
}

function attachProviderBlockHandlers(block, providerId) {
  const settingsButton = block.querySelector("[data-provider-settings-button]");
  const settingsDropdown = block.querySelector("[data-provider-settings-dropdown]");

  settingsButton.addEventListener("click", (event) => {
    event.stopPropagation();
    const shouldOpen = settingsDropdown.hidden;
    closeAllProviderSettingsMenus();
    settingsDropdown.hidden = !shouldOpen;
    settingsButton.setAttribute("aria-expanded", String(shouldOpen));
  });

  settingsDropdown.addEventListener("click", (event) => {
    event.stopPropagation();
  });

  for (const option of block.querySelectorAll("[data-frequency-option]")) {
    option.addEventListener("click", () => {
      setProviderInterval(providerId, option.dataset.frequencyOption, refreshSingleProvider);
      syncFrequencyOptions(block, option.dataset.frequencyOption);
      updateProviderUpdateTimeText(
        block,
        providerDataCache.get(providerId) ?? { pending: true },
        getProviderNextRefreshAt(providerId),
      );
    });
  }

  block.querySelector("[data-manual-refresh]")?.addEventListener("click", () => {
    refreshSingleProvider(providerId);
  });
  attachSectionHandlers(block);
}

function attachDataErrorsBlockHandlers(block) {
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

function attachPlanLinkHandlers(block) {
  for (const link of block.querySelectorAll("[data-plan-link-url]")) {
    link.addEventListener("click", () => {
      openExternalUrl(link.dataset.planLinkUrl);
    });
  }
}

function attachSectionHandlers(block) {
  attachDataErrorsBlockHandlers(block);
  attachCliAuthorizationHandlers(block);
  attachPlanLinkHandlers(block);
}

// Display toggles (Show limits / Show plan) never trigger a
// refresh: they re-render every already-mounted provider block from the
// data already held in providerDataCache, instantly and without a backend
// call. See handleDisplaySettingsChange in settings.js.
export function refreshProviderSectionsFromCache() {
  for (const provider of providerDataCache.values()) {
    const block = getProviderBlock(provider.id);
    if (!block) {
      continue;
    }

    updateProviderBlockData(block, provider, getProviderNextRefreshAt(provider.id));
    attachSectionHandlers(block);
  }

  scheduleSectionSlotAlignment();
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
  restartProviderRefreshTimer(provider.id, refreshSingleProvider);
  const block = renderProvider(provider, getProviderInterval(provider.id), getProviderNextRefreshAt(provider.id));
  attachProviderBlockHandlers(block, provider.id);
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
    getProviderBlock(providerId)?.remove();
  }

  scheduleSectionSlotAlignment();
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

  scheduleSectionSlotAlignment();
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

// A failed fetch previously rejected silently: providerRefreshInFlight was
// still cleared in `finally`, but nothing told the user the click did
// anything. Manual and scheduled refreshes share this function, so surfacing
// the error here covers both.
function setManualRefreshLoading(providerId, isLoading) {
  const button = getProviderBlock(providerId)?.querySelector("[data-manual-refresh]");
  if (!button) {
    return;
  }

  button.disabled = isLoading;
  button.textContent = isLoading ? "UPDATING…" : "UPDATE NOW";
}

async function refreshSingleProvider(providerId) {
  if (!isProviderEnabled(providerId) || providerRefreshInFlight.has(providerId)) {
    return;
  }

  providerRefreshInFlight.add(providerId);
  setManualRefreshLoading(providerId, true);

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
      updateProviderBlockData(block, provider, getProviderNextRefreshAt(providerId));
      attachSectionHandlers(block);
    }
  } catch (error) {
    setErrorState(error?.message || String(error) || `Could not refresh ${providerId}.`);
  } finally {
    setManualRefreshLoading(providerId, false);
    scheduleSectionSlotAlignment();
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
    scheduleSectionSlotAlignment();
  }

  for (const providerId of enabledProviders) {
    refreshSingleProvider(providerId);
  }
}
