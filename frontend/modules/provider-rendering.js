import { updateFrequencyOptions } from "./constants.js";
import { colorForRemaining, escapeHtml, formatDecimal, formatSourceStatusLine, formatTimestampForDisplay, formatUpdateTimeLine } from "./provider-formatters.js";
import { isShowLimitsEnabled, isShowPlanEnabled, isShowSourceEnabled, isShowUpdateTimeEnabled } from "./settings.js";

const FREQUENCY_ICON_BADGES = {
  "Manual only": "X",
  "1 min": "1",
  "5 min": "5",
  "10 min": "10",
  "30 min": "30",
  "1 hour": "60",
};

// Single glyph shared by the header trigger, the frequency options, and the
// UPDATE NOW row: a refresh-cw arrow loop with an optional badge (minutes,
// or "X" for manual-only) lettered into the open space at its center. The
// arrows trace a wide ring near the icon's edge, so the center stays clear
// almost to the icon's full radius — the badge can run large without
// touching the strokes. Font size is set in viewBox units, so it scales
// automatically with whatever pixel size the caller renders the icon at.
export function buildRefreshCwIconSvg(badge, { size = 20 } = {}) {
  const badgeMarkup = badge
    ? `<text class="refresh-icon-badge" x="12" y="12.5" text-anchor="middle" dominant-baseline="middle" font-size="${badge.length > 1 ? 9.5 : 13}" font-weight="700" stroke="none" fill="currentColor">${escapeHtml(badge)}</text>`
    : "";
  return `
    <svg class="settings-icon refresh-icon" xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
      <path d="M21 3v5h-5" />
      <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
      <path d="M8 16H3v5" />
      ${badgeMarkup}
    </svg>
  `;
}

function providerLabel(providerId) {
  return providerId.charAt(0).toUpperCase() + providerId.slice(1);
}

export function createEmptyProvider(providerId, selectedUpdateFrequency) {
  return {
    id: providerId,
    label: providerLabel(providerId),
    limits: [],
    availableLimitResets: null,
    plan: { lines: [], links: [] },
    sourceId: null,
    dataTimestamp: null,
    selectedUpdateFrequency,
    errorMessage: null,
    noFreshData: false,
    authorizationRequired: null,
    pending: true,
  };
}

// Key-value lines are emitted as three spans — label, separator, value —
// instead of one flat string. The Main Window concatenates them back into
// exactly the "Available credits: 23.8" line it always showed (the spans are
// inline and carry no styling of their own there), while the Popover lays
// them out as a label-left/value-right menu row and hides the separator, the
// way a system panel writes a key-value pair. See
// docs/desktop/mac-popover.md#visual-layer.
function buildKeyValueHtml(label, value) {
  return `<span class="kv-label">${escapeHtml(label)}</span><span class="kv-separator">: </span><span class="kv-value">${escapeHtml(value)}</span>`;
}

function formatCreditsValue(provider) {
  if (provider.creditsRemaining == null) {
    return "";
  }

  return formatDecimal(provider.creditsRemaining);
}

function buildLimitResetsHtml(provider) {
  if (Number(provider.availableLimitResets) <= 0) {
    return "";
  }

  return `
    <p class="credits-info">${buildKeyValueHtml("Available resets", String(provider.availableLimitResets))}</p>
  `;
}

function cliAuthorizationCopy(providerKey) {
  if (providerKey === "claude") {
    return { message: "You\u2019re not signed in to Claude CLI.", signInLabel: "Sign in to Claude", loginCommand: "claude login" };
  }

  return { message: "You\u2019re not signed in to Codex CLI.", signInLabel: "Sign in to Codex", loginCommand: "codex login" };
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
      <p>No fresh limits' data.</p>
      <button type="button" class="provider-link" data-open-data-errors>
        More details
      </button>
    </div>
  `;
}

function buildLimitRowsHtml(provider) {
  if (!provider.limits.length) {
    if (provider.pending || provider.availableLimitResets != null) {
      return "";
    }

    if (provider.authorizationRequired) {
      return buildCliAuthorizationHtml(provider.authorizationRequired);
    }

    if (provider.noFreshData) {
      return buildNoFreshDataHtml();
    }

    const message = escapeHtml(provider.errorMessage || "No usable limit records from this source");
    const details = provider.errorMessage === "Local provider data is outdated" ? `<button type="button" class="provider-link" data-open-data-errors>More details</button>` : "";
    return `<div><p class="provider-message">${message}</p>${details}</div>`;
  }

  return provider.limits.map((limit) => {
    const remaining = Number(limit.remainingPercentage) || 0;
    const percent = remaining.toFixed(1);
    const width = Math.max(0, Math.min(100, remaining));
    const formattedResetTime = formatTimestampForDisplay(limit.resetTime);
    const resetText = formattedResetTime ? `reset ${escapeHtml(formattedResetTime)}` : "";

    return `
      <div class="limit-row">
        <div class="limit-top"><span class="limit-label">${escapeHtml(limit.label)}</span><span class="limit-separator"> | </span><span class="limit-value">${percent}% left</span></div>
        <progress class="meter" data-remaining="${width}" value="${width}" max="100" aria-label="${escapeHtml(provider.label)} ${escapeHtml(limit.label)} ${percent}% left"></progress>
        ${resetText ? `<div class="limit-reset">${resetText}</div>` : ""}
      </div>
    `;
  }).join("");
}

// CSP forbids `style="..."` attributes (no `unsafe-inline`), so the
// interpolated fill color can't be written into the HTML string above.
// Setting individual CSSOM properties via `style.setProperty` is not an
// inline-style attribute and is unaffected by that restriction.
function applyMeterFillColors(root) {
  root.querySelectorAll(".meter[data-remaining]").forEach((meter) => {
    meter.style.setProperty("--meter-fill", colorForRemaining(Number(meter.dataset.remaining)));
  });
}

const SECTION_HEADINGS = {
  limits: "LIMITS",
  plan: "PLAN",
};

// A section slot is always rendered when its display toggle is on. Content
// adds the divider heading and body; an empty slot stays blank and is later
// sized to match the tallest visible slot of the same kind. While the
// provider is still pending (cold start / not yet fetched), the heading is
// known from the toggle alone, so it renders immediately with a placeholder
// body reserving roughly the space real content will need — this is what
// keeps the card from collapsing to nothing before the first fetch resolves.
function buildProviderSectionHtml(kind, bodyHtml, pending) {
  const heading = `<h3 class="section-heading"><span class="section-heading-label">${escapeHtml(SECTION_HEADINGS[kind])}</span></h3>`;

  if (bodyHtml) {
    return `
      <div class="provider-section provider-section--${kind}" data-section-slot="${kind}">
        ${heading}<div class="provider-section-body">${bodyHtml}</div>
      </div>
    `;
  }

  if (pending) {
    return `
      <div class="provider-section provider-section--${kind}" data-section-slot="${kind}">
        ${heading}<div class="provider-section-body provider-section-body--placeholder" aria-hidden="true"></div>
      </div>
    `;
  }

  return `
    <div class="provider-section provider-section--${kind}" data-section-slot="${kind}"></div>
  `;
}

function buildLimitsBodyHtml(provider) {
  const limitRowsHtml = buildLimitRowsHtml(provider);
  const creditsValue = formatCreditsValue(provider);
  const limitResetsHtml = buildLimitResetsHtml(provider);

  if (!limitRowsHtml && !creditsValue && !limitResetsHtml) {
    return "";
  }

  return `
    <div class="limits">${limitRowsHtml}</div>
    <p class="credits-info" ${creditsValue ? "" : "hidden"}>${buildKeyValueHtml("Available credits", creditsValue)}</p>
    <div class="limit-resets-slot">${limitResetsHtml}</div>
  `;
}

function buildPlanLinesHtml(plan) {
  return (plan?.lines ?? []).map((line) => `<p class="section-line">${escapeHtml(line)}</p>`).join("");
}

// The management links share one line, joined by a middot, per
// docs/desktop/ui/provider-block-content.md. A single link is rendered
// without a separator. Each link stays a button so the click keeps going
// through openExternalUrl rather than navigating the app window.
function buildPlanLinksHtml(plan) {
  const links = (plan?.links ?? []).filter((link) => link?.url && link?.label);

  if (!links.length) {
    return "";
  }

  const linksHtml = links.map((link) => `<button type="button" class="provider-link provider-link--external" data-plan-link-url="${escapeHtml(link.url)}">${escapeHtml(link.label)}</button>`)
    .join(`<span class="plan-links-separator" aria-hidden="true">·</span>`);

  return `<p class="plan-links">${linksHtml}</p>`;
}

function buildPlanBodyHtml(provider) {
  return `${buildPlanLinesHtml(provider.plan)}${buildPlanLinksHtml(provider.plan)}`;
}

function buildProviderSectionsHtml(provider) {
  return [
    isShowLimitsEnabled() ? buildProviderSectionHtml("limits", buildLimitsBodyHtml(provider), provider.pending) : "",
    isShowPlanEnabled() ? buildProviderSectionHtml("plan", buildPlanBodyHtml(provider), provider.pending) : "",
  ].join("");
}

function buildFrequencyOptionsHtml(selectedUpdateFrequency) {
  return updateFrequencyOptions
    .map((option) => `
      <button type="button" class="frequency-option" data-frequency-option="${escapeHtml(option)}" aria-pressed="${option === selectedUpdateFrequency}">
        ${buildRefreshCwIconSvg(FREQUENCY_ICON_BADGES[option] ?? "", { size: 18 })}<span class="frequency-option-label">${escapeHtml(option)}</span>
      </button>
    `)
    .join("");
}

function buildSourceLineHtml(provider) {
  return `<p class="source-line" ${isShowSourceEnabled() ? "" : "hidden"}>${escapeHtml(formatSourceStatusLine(provider))}</p>`;
}

function buildUpdateTimeLineHtml(provider, nextRefreshAt) {
  return `<p class="update-time-line" ${isShowUpdateTimeEnabled() ? "" : "hidden"}>${escapeHtml(formatUpdateTimeLine(provider, nextRefreshAt))}</p>`;
}

export function renderProvider(provider, selectedUpdateFrequency, nextRefreshAt) {
  const block = document.createElement("article");
  block.className = "provider-block";
  block.dataset.providerId = provider.id;
  const settingsDropdownId = `provider-settings-dropdown-${provider.id}`;

  block.innerHTML = `
    <div class="provider-refresh-glare" aria-hidden="true"></div>
    <div class="provider-content">
      <div class="provider-header">
        <h2>${escapeHtml(provider.label)}</h2>
        <div class="provider-settings-menu" data-provider-settings-menu>
          <button
            type="button"
            class="settings-button provider-settings-button"
            data-provider-settings-button
            aria-label="${escapeHtml(provider.label)} update settings"
            aria-haspopup="true"
            aria-expanded="false"
            aria-controls="${settingsDropdownId}"
          >${buildRefreshCwIconSvg(FREQUENCY_ICON_BADGES[selectedUpdateFrequency] ?? "", { size: 20 })}</button>
          <div class="settings-dropdown provider-settings-dropdown" id="${settingsDropdownId}" data-provider-settings-dropdown hidden>
            <div class="settings-section">
              <button type="button" class="frequency-option manual-refresh-option" data-manual-refresh>
                ${buildRefreshCwIconSvg("", { size: 18 })}<span class="frequency-option-label">UPDATE NOW</span>
              </button>
            </div>
            <div class="settings-section">
              <p class="settings-section-label">UPDATE FREQUENCY</p>
              <div class="frequency-options" role="group" aria-label="${escapeHtml(provider.label)} update frequency">
                ${buildFrequencyOptionsHtml(selectedUpdateFrequency)}
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="provider-sections">${buildProviderSectionsHtml(provider)}</div>
    </div>
    <div class="provider-footer">
      ${buildUpdateTimeLineHtml(provider, nextRefreshAt)}
      ${buildSourceLineHtml(provider)}
    </div>
  `;
  applyMeterFillColors(block);

  return block;
}

export function updateProviderBlockData(block, provider, nextRefreshAt) {
  const sections = block.querySelector(".provider-sections");
  sections.innerHTML = buildProviderSectionsHtml(provider);
  applyMeterFillColors(sections);
  block.querySelector(".source-line").outerHTML = buildSourceLineHtml(provider);
  block.querySelector(".update-time-line").outerHTML = buildUpdateTimeLineHtml(provider, nextRefreshAt);
}

export function updateProviderUpdateTimeText(block, provider, nextRefreshAt) {
  const el = block.querySelector(".update-time-line");
  if (el) {
    el.textContent = formatUpdateTimeLine(provider, nextRefreshAt);
  }
}

export function syncFrequencyOptions(block, selectedUpdateFrequency) {
  for (const option of block.querySelectorAll("[data-frequency-option]")) {
    option.setAttribute("aria-pressed", String(option.dataset.frequencyOption === selectedUpdateFrequency));
  }

  const settingsButton = block.querySelector("[data-provider-settings-button]");
  if (settingsButton) {
    settingsButton.innerHTML = buildRefreshCwIconSvg(FREQUENCY_ICON_BADGES[selectedUpdateFrequency] ?? "", { size: 20 });
  }
}
