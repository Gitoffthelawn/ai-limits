import { DEFAULT_UPDATE_FREQUENCY, PROVIDER_IDS, PROVIDER_INTERVALS_STORAGE_KEY, updateFrequencyOptions } from "./constants.js";
import { isProviderEnabled } from "./settings.js";

const providerRefreshIntervals = new Map();
const providerRefreshTimers = new Map();
const providerNextRefreshAt = new Map();
// Epoch ms of this provider's last actual collection. Seeded from the
// backend's `collectedAt` (raw ISO, unlike `dataTimestamp` which arrives
// pre-formatted for display) whenever it is known, so the schedule is based
// on the shared collection instant rather than whenever this particular
// window happened to observe a fetch resolve — both Main Window and Popover
// compute the same next-refresh target from the same collection. Falls back
// to this call's own Date.now() only when no `collectedAt` is available
// (a failed fetch, or a snapshot that predates this field).
const providerLastUpdateAt = new Map();

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

  clearTimeout(timerId);
  providerRefreshTimers.delete(providerId);
}

// Marks `providerId`'s last update instant. Pass the backend's raw
// `collectedAt` when available, so the schedule is anchored to the actual
// collection time shared by every surface; pass nothing (or an unparseable
// value) to fall back to this call's own Date.now() — used as the retry
// anchor after a failed fetch, where no collection happened at all: an
// attempt just happened either way, so the next one is a full interval out
// rather than an immediate hot loop. Does not itself (re)schedule anything;
// pair with restartProviderRefreshTimer.
export function recordProviderUpdateNow(providerId, collectedAt) {
  const parsed = collectedAt ? Date.parse(collectedAt) : NaN;
  providerLastUpdateAt.set(providerId, Number.isNaN(parsed) ? Date.now() : parsed);
}

// The next automatic refresh is always one interval after the provider's
// last recorded update (via recordProviderUpdateNow), never from whenever
// this function happened to run — so switching frequency, a manual refresh,
// or re-enabling a provider all resolve to the same target time instead of
// each restarting its own countdown. If no update has ever been recorded, or
// the computed target time has already passed (e.g. the frequency just
// changed to something shorter than the time since the last update), this
// refreshes immediately rather than waiting out a stale interval.
export function restartProviderRefreshTimer(providerId, refreshProvider) {
  stopProviderRefreshTimer(providerId);

  if (!isProviderEnabled(providerId)) {
    providerNextRefreshAt.set(providerId, null);
    return;
  }

  const intervalMs = frequencyToMs(getProviderInterval(providerId));
  if (intervalMs == null) {
    providerNextRefreshAt.set(providerId, null);
    return;
  }

  const lastUpdateMs = providerLastUpdateAt.get(providerId) ?? null;
  if (lastUpdateMs == null) {
    providerNextRefreshAt.set(providerId, Date.now());
    refreshProvider(providerId);
    return;
  }

  const nextRefreshAt = lastUpdateMs + intervalMs;
  providerNextRefreshAt.set(providerId, nextRefreshAt);

  const delay = nextRefreshAt - Date.now();
  if (delay <= 0) {
    refreshProvider(providerId);
    return;
  }

  const timerId = setTimeout(() => refreshProvider(providerId), delay);
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
