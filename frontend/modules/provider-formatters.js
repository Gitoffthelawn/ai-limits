const MONTH_NAMES = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const SOURCE_DISPLAY_LABELS = {
  "codex-local": "Local files",
  "codex-rpc": "RPC",
  "codex-cli": "CLI",
  "claude-rpc": "RPC",
  "claude-cli": "CLI",
  "claude-local": "Local files",
  "cursor-api2": "API2",
};

function stripTimezoneSuffix(value) {
  return String(value)
    .replace(/\s*\([^()]*\)\s*$/, "")
    .replace(/\s*UTC[+-]\d{1,2}(:\d{2})?\s*$/i, "")
    .replace(/\s*GMT[+-]\d{1,2}(:\d{2})?\s*$/i, "")
    .replace(/([+-]\d{2}:\d{2}|Z)$/, "")
    .trim();
}

export function formatTimestampForDisplay(value) {
  if (value == null || value === "") return null;
  const date = toDate(value);
  if (!date) return stripTimezoneSuffix(value);
  const time = `${pad2(date.getHours())}:${pad2(date.getMinutes())}`;
  if (isSameLocalDay(date, new Date())) return time;
  return `${MONTH_NAMES[date.getMonth()]} ${date.getDate()}, ${time}`;
}

export function formatSourceIdLine(provider) {
  if (provider.pending) return "—";
  return `Source ${SOURCE_DISPLAY_LABELS[provider.sourceId] ?? provider.sourceId ?? "Unknown"},`;
}

export function formatSourceTimestampLine(provider) {
  if (provider.pending) return "—";
  return `as of ${formatTimestampForDisplay(provider.dataTimestamp) || "unknown"}`;
}

export function formatSourceStatusLine(provider) {
  if (provider.pending) return "—";
  return `${formatSourceIdLine(provider)} ${formatSourceTimestampLine(provider)}`;
}

function formatTimeOnly(value) {
  if (value == null || value === "") return null;
  const date = toDate(value);
  if (date) return `${pad2(date.getHours())}:${pad2(date.getMinutes())}`;
  // Backend timestamps arrive pre-formatted for display (e.g. "20:03" or
  // "Aug 3, 20:03"), not as parseable instants, so fall back to pulling the
  // trailing HH:MM out of the already-formatted string.
  const match = String(value).match(/(\d{1,2}):(\d{2})\s*$/);
  return match ? `${pad2(match[1])}:${match[2]}` : null;
}

export function formatUpdateTimeLine(provider, nextRefreshAt) {
  if (provider.pending) return "—";
  const last = formatTimeOnly(provider.dataTimestamp) ?? "unknown";
  const next = formatTimeOnly(nextRefreshAt) ?? "Manual only";
  return `Last upd ${last}, next ${next}`;
}

export function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function formatDecimal(value) {
  const rounded = Math.round(Number(value) * 10) / 10;
  return Number.isInteger(rounded) ? String(rounded) : rounded.toFixed(1);
}

export function parseTimestampMs(value) {
  const date = toDate(value);
  return date ? date.getTime() : null;
}

function toDate(value) {
  if (value instanceof Date) return Number.isNaN(value.getTime()) ? null : value;
  if (typeof value === "number") {
    const date = new Date(value < 1e12 ? value * 1000 : value);
    return Number.isNaN(date.getTime()) ? null : date;
  }
  if (typeof value !== "string" || !value.trim()) return null;
  const trimmed = value.trim();
  const numeric = /^\d+$/.test(trimmed) ? Number(trimmed) : null;
  const date = numeric == null ? new Date(trimmed) : new Date(trimmed.length > 10 ? numeric : numeric * 1000);
  return Number.isNaN(date.getTime()) ? null : date;
}

function isSameLocalDay(a, b) {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

function pad2(value) {
  return String(value).padStart(2, "0");
}
