import { updateFrequencyOptions } from "./constants.js";
import { escapeHtml, formatDecimal, formatSourceStatusLine, formatTimestampForDisplay, formatUpdateTimeLine } from "./provider-formatters.js";
import { isShowLimitsEnabled, isShowPlanEnabled, isShowSourceEnabled, isShowUpdateTimeEnabled } from "./settings.js";

const FREQUENCY_ICON_BADGES = {
  "Manual only": "X",
  "1 min": "1",
  "5 min": "5",
  "10 min": "10",
  "30 min": "30",
  "1 hour": "60",
};

function buildFrequencyIconSvg(option) {
  const badge = FREQUENCY_ICON_BADGES[option] ?? "";
  const badgeFontSize = badge.length > 1 ? 7 : 8.5;
  return `
    <svg class="settings-icon frequency-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="10" cy="10" r="8" />
      <path d="M10 5.5v4.5l3 1.5" />
      <circle class="frequency-icon-badge-chip" cx="18" cy="18" r="6.5" stroke-width="1.5" />
      <text class="frequency-icon-badge" x="18" y="18.5" text-anchor="middle" dominant-baseline="middle" font-size="${badgeFontSize}" font-weight="700" stroke="none" fill="currentColor">${escapeHtml(badge)}</text>
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
    const meterTone = width <= 1 ? "danger" : width <= 50 ? "warning" : "success";
    const formattedResetTime = formatTimestampForDisplay(limit.resetTime);
    const resetText = formattedResetTime ? `reset ${escapeHtml(formattedResetTime)}` : "";

    return `
      <div class="limit-row">
        <div class="limit-top">${escapeHtml(limit.label)} | ${percent}% left</div>
        <progress class="meter meter--${meterTone}" value="${width}" max="100" aria-label="${escapeHtml(provider.label)} ${escapeHtml(limit.label)} ${percent}% left"></progress>
        ${resetText ? `<div class="limit-reset">${resetText}</div>` : ""}
      </div>
    `;
  }).join("");
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
  const creditsLine = formatCreditsLine(provider);
  const limitResetsHtml = buildLimitResetsHtml(provider);

  if (!limitRowsHtml && !creditsLine && !limitResetsHtml) {
    return "";
  }

  return `
    <div class="limits">${limitRowsHtml}</div>
    <p class="credits-info" ${creditsLine ? "" : "hidden"}>${escapeHtml(creditsLine)}</p>
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
        ${buildFrequencyIconSvg(option)}<span class="frequency-option-label">${escapeHtml(option)}</span>
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
    <div class="provider-content">
      <div class="provider-header">
        <h2>${escapeHtml(provider.label)}</h2>
      </div>
      <div class="provider-sections">${buildProviderSectionsHtml(provider)}</div>
    </div>
    <div class="provider-footer">
      ${buildSourceLineHtml(provider)}
      <div class="provider-actions">
        <button type="button" class="provider-manual-refresh" data-manual-refresh>
          UPDATE NOW
        </button>
        <div class="provider-settings-menu" data-provider-settings-menu>
          <button
            type="button"
            class="settings-button provider-settings-button"
            data-provider-settings-button
            aria-label="${escapeHtml(provider.label)} update settings"
            aria-haspopup="true"
            aria-expanded="false"
            aria-controls="${settingsDropdownId}"
          >${buildFrequencyIconSvg(selectedUpdateFrequency)}</button>
          <div class="settings-dropdown provider-settings-dropdown" id="${settingsDropdownId}" data-provider-settings-dropdown hidden>
            <div class="settings-section">
              <p class="settings-section-label">UPDATE FREQUENCY</p>
              <div class="frequency-options" role="group" aria-label="${escapeHtml(provider.label)} update frequency">
                ${buildFrequencyOptionsHtml(selectedUpdateFrequency)}
              </div>
            </div>
          </div>
        </div>
      </div>
      ${buildUpdateTimeLineHtml(provider, nextRefreshAt)}
    </div>
  `;

  return block;
}

export function updateProviderBlockData(block, provider, nextRefreshAt) {
  block.querySelector(".provider-sections").innerHTML = buildProviderSectionsHtml(provider);
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
    settingsButton.innerHTML = buildFrequencyIconSvg(selectedUpdateFrequency);
  }
}
