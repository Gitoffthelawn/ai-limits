import { updateFrequencyOptions } from "./constants.js";
import { escapeHtml, formatDecimal, formatNextUpdateLine, formatSourceStatusLine, formatTimestampForDisplay } from "./provider-formatters.js";
import { buildSourcePriorityControlHtml, isShowLimitsEnabled, isShowPlanEnabled } from "./settings.js";

const GEAR_ICON_SVG = `
  <svg class="settings-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
    <circle cx="12" cy="12" r="3" />
  </svg>
`;

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
  plan: "SUBSCRIPTION",
};

// A section slot is always rendered when its display toggle is on. Content
// adds the divider heading and body; an empty slot stays blank and is later
// sized to match the tallest visible slot of the same kind.
function buildProviderSectionHtml(kind, bodyHtml) {
  return `
    <div class="provider-section provider-section--${kind}" data-section-slot="${kind}">
      ${bodyHtml ? `<h3 class="section-heading"><span class="section-heading-label">${escapeHtml(SECTION_HEADINGS[kind])}</span></h3><div class="provider-section-body">${bodyHtml}</div>` : ""}
    </div>
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
    isShowLimitsEnabled() ? buildProviderSectionHtml("limits", buildLimitsBodyHtml(provider)) : "",
    isShowPlanEnabled() ? buildProviderSectionHtml("plan", buildPlanBodyHtml(provider)) : "",
  ].join("");
}

function buildFrequencyOptionsHtml(selectedUpdateFrequency) {
  return updateFrequencyOptions
    .map((option) => `<button type="button" class="frequency-option" data-frequency-option="${escapeHtml(option)}" aria-pressed="${option === selectedUpdateFrequency}">${escapeHtml(option)}</button>`)
    .join("");
}

function buildSourceInfoHtml(provider, nextRefreshAt) {
  return `
    <span class="source-status">${escapeHtml(formatSourceStatusLine(provider))}</span>
    <span class="source-next-update">${escapeHtml(formatNextUpdateLine(nextRefreshAt))}</span>
  `;
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
      <p class="source-info">${buildSourceInfoHtml(provider, nextRefreshAt)}</p>
    </div>
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
        >${GEAR_ICON_SVG}</button>
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
  `;

  return block;
}

export function updateProviderBlockData(block, provider, nextRefreshAt) {
  block.querySelector(".provider-sections").innerHTML = buildProviderSectionsHtml(provider);
  block.querySelector(".source-info").innerHTML = buildSourceInfoHtml(provider, nextRefreshAt);
}

export function updateProviderNextUpdateText(block, nextRefreshAt) {
  const el = block.querySelector(".source-next-update");
  if (el) {
    el.textContent = formatNextUpdateLine(nextRefreshAt);
  }
}

export function syncFrequencyOptions(block, selectedUpdateFrequency) {
  for (const option of block.querySelectorAll("[data-frequency-option]")) {
    option.setAttribute("aria-pressed", String(option.dataset.frequencyOption === selectedUpdateFrequency));
  }
}
