import { updateFrequencyOptions } from "./constants.js";
import { escapeHtml, formatDecimal, formatSourceIdLine, formatSourceTimestampLine, formatTimestampForDisplay } from "./provider-formatters.js";
import { buildSourcePriorityControlHtml, isShowLimitsEnabled, isShowPlanEnabled, isShowUsageEnabled } from "./settings.js";

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
    usage: { lines: [] },
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

// Limits, Plan, and Usage are the desktop rendering of the product's three
// output kinds (see docs/product/output-kinds.md). Each is shown only when
// it has content and only when its display toggle is on; they are separated
// from each other by a divider that appears only between two visible
// sections, never leading, trailing, or next to a hidden one.

function buildLimitsSectionHtml(provider) {
  const limitRowsHtml = buildLimitRowsHtml(provider);
  const creditsLine = formatCreditsLine(provider);
  const limitResetsHtml = buildLimitResetsHtml(provider);

  if (!limitRowsHtml && !creditsLine && !limitResetsHtml) {
    return "";
  }

  return `
    <div class="provider-section provider-section--limits">
      <div class="limits">${limitRowsHtml}</div>
      <p class="credits-info" ${creditsLine ? "" : "hidden"}>${escapeHtml(creditsLine)}</p>
      <div class="limit-resets-slot">${limitResetsHtml}</div>
    </div>
  `;
}

function buildPlanLinesHtml(plan) {
  return (plan?.lines ?? []).map((line) => `<p class="section-line">${escapeHtml(line)}</p>`).join("");
}

function buildPlanLinksHtml(plan) {
  return (plan?.links ?? []).map((link) => `
    <button type="button" class="provider-link provider-link--external" data-plan-link-url="${escapeHtml(link.url)}">
      ${escapeHtml(link.label)}
    </button>
  `).join("");
}

function buildPlanSectionHtml(provider) {
  const linesHtml = buildPlanLinesHtml(provider.plan);
  const linksHtml = buildPlanLinksHtml(provider.plan);

  if (!linesHtml && !linksHtml) {
    return "";
  }

  return `<div class="provider-section provider-section--plan">${linesHtml}${linksHtml}</div>`;
}

function buildUsageSectionHtml(provider) {
  const linesHtml = (provider.usage?.lines ?? []).map((line) => `<p class="section-line">${escapeHtml(line)}</p>`).join("");

  if (!linesHtml) {
    return "";
  }

  return `<div class="provider-section provider-section--usage">${linesHtml}</div>`;
}

function buildProviderSectionsHtml(provider) {
  const sections = [
    isShowLimitsEnabled() ? buildLimitsSectionHtml(provider) : "",
    isShowPlanEnabled() ? buildPlanSectionHtml(provider) : "",
    isShowUsageEnabled() ? buildUsageSectionHtml(provider) : "",
  ].filter(Boolean);

  return sections.join(`<hr class="section-divider" aria-hidden="true">`);
}

export function renderProvider(provider, selectedUpdateFrequency) {
  const block = document.createElement("article");
  block.className = "provider-block";
  block.dataset.providerId = provider.id;
  const frequencyOptions = updateFrequencyOptions.map((option) => `<option ${option === selectedUpdateFrequency ? "selected" : ""}>${option}</option>`).join("");

  block.innerHTML = `
    <div class="provider-status" hidden aria-live="polite">
      <span class="provider-status-indicator" aria-hidden="true"></span>
      <span class="provider-status-text"></span>
    </div>
    <div class="provider-content">
      <div class="provider-header">
        <h2>${escapeHtml(provider.label)}</h2>
      </div>
      <div class="provider-sections">${buildProviderSectionsHtml(provider)}</div>
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

export function updateProviderBlockData(block, provider) {
  block.querySelector(".provider-sections").innerHTML = buildProviderSectionsHtml(provider);
  block.querySelector(".source-id").textContent = formatSourceIdLine(provider);
  block.querySelector(".source-timestamp").textContent = formatSourceTimestampLine(provider);
}
