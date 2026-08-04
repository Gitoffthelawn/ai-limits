import { DEFAULT_UPDATE_FREQUENCY, PROVIDER_IDS, PROVIDER_INTERVALS_STORAGE_KEY, updateFrequencyOptions } from "./constants.js";
import { isProviderEnabled } from "./settings.js";

const providerRefreshIntervals = new Map();
const providerRefreshTimers = new Map();
const providerNextRefreshAt = new Map();

function normalizeUpdateFrequency(frequency) {
  if (typeof frequency !== "string") {
    return DEFAULT_UPDATE_FREQUENCY;
  }

  return updateFrequencyOptions.includes(frequency) ? frequency : DEFAULT_UPDATE_FREQUENCY;
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
    providerRefreshIntervals.set(providerId, stored[providerId] ?? DEFAULT_UPDATE_FREQUENCY);
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

export function getProviderInterval(providerId) {
  return providerRefreshIntervals.get(providerId) ?? DEFAULT_UPDATE_FREQUENCY;
}

// null means the next scheduled refresh is unknown or there is none (manual only).
export function getProviderNextRefreshAt(providerId) {
  return providerNextRefreshAt.get(providerId) ?? null;
}

export function ensureProviderInterval(providerId, fallbackFrequency) {
  if (!providerRefreshIntervals.has(providerId)) {
    providerRefreshIntervals.set(providerId, normalizeUpdateFrequency(fallbackFrequency));
  }
}

export function stopProviderRefreshTimer(providerId) {
  const timerId = providerRefreshTimers.get(providerId);
  if (timerId == null) {
    return;
  }

  clearInterval(timerId);
  providerRefreshTimers.delete(providerId);
}

export function restartProviderRefreshTimer(providerId, refreshProvider) {
  stopProviderRefreshTimer(providerId);

  if (!isProviderEnabled(providerId)) {
    providerNextRefreshAt.set(providerId, null);
    return;
  }

  const intervalMs = frequencyToMs(getProviderInterval(providerId));
  providerNextRefreshAt.set(providerId, intervalMs == null ? null : Date.now() + intervalMs);
  if (intervalMs == null) {
    return;
  }

  const timerId = setInterval(() => {
    providerNextRefreshAt.set(providerId, Date.now() + intervalMs);
    refreshProvider(providerId);
  }, intervalMs);
  providerRefreshTimers.set(providerId, timerId);
}

export function setProviderInterval(providerId, frequency, refreshProvider) {
  if (!updateFrequencyOptions.includes(frequency)) {
    return;
  }

  providerRefreshIntervals.set(providerId, frequency);
  saveProviderIntervals();
  restartProviderRefreshTimer(providerId, refreshProvider);
}
