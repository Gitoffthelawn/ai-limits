// Shared chrome (top bar, view tabs, footer) for the macOS Menu Bar Popover,
// per docs/desktop/mac-popover.md#toolbar and #view-tabs.
//
// This module is imported by both:
//   - popover.js — the real Popover surface, driven by the real provider
//     data pipeline (providers.js) and real Tauri IPC.
//   - showcase.js — the `?showcase=popover` browser preview, driven by mock
//     SHOWCASE_PROVIDERS data.
// so the two never render or filter differently. Keep this module free of any
// dependency on showcase-only or real-pipeline-only state — it only knows
// about provider ids/labels handed to it by the caller.
//
// Icons are hand-authored here rather than borrowed from the Main Window's
// glyph builders: the popover draws at the system-panel scale (15px, 1.5
// stroke, currentColor, muted opacity), not the Main Window's 18-20px/2.0.
import { escapeHtml } from "./provider-formatters.js";

function buildPopoverIconSvg(paths) {
  return `
    <svg class="popover-icon" xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>
  `;
}

// A single circular refresh arrow — the system-standard "refresh everything"
// shape, drawn directly instead of reshaping another module's markup.
export const POPOVER_UPDATE_ALL_ICON_SVG = buildPopoverIconSvg(`
    <path d="M20.5 12a8.5 8.5 0 1 1-2.49-6.01" />
    <path d="M20.5 4.5V10h-5.5" />
`);

// Lucide "info": a circle with a lower stem and a dot — the same glyph the
// Main Window's Help entry point carries, redrawn at popover weight.
export const POPOVER_INFO_ICON_SVG = buildPopoverIconSvg(`
    <circle cx="12" cy="12" r="9" />
    <path d="M12 16.5v-5" />
    <path d="M12 8h.01" />
`);

// Lucide "settings" gear, likewise redrawn at popover weight.
export const POPOVER_GEAR_ICON_SVG = buildPopoverIconSvg(`
    <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
    <circle cx="12" cy="12" r="3" />
`);

function capitalizeProviderId(providerId) {
  return providerId.charAt(0).toUpperCase() + providerId.slice(1);
}

// `providerIds` is the already-filtered list of providers whose tab should
// appear — per docs/desktop/mac-popover.md#view-tabs, a disabled provider's
// tab does not appear. `All` always appears regardless.
//
// Rendered as a macOS segmented control: `role="tab"` inside the
// `role="tablist"`, selection carried by `aria-selected` (the role's own
// state) rather than the button-flavored `aria-pressed`.
export function buildPopoverTabsHtml(providerIds) {
  const providerTabsHtml = providerIds.map((providerId) => `
    <button type="button" class="popover-tab" role="tab" data-popover-tab="${escapeHtml(providerId)}" aria-selected="false">${escapeHtml(capitalizeProviderId(providerId))}</button>
  `).join("");

  return `
    <div class="popover-tabs" role="tablist" aria-label="Provider view">
      <button type="button" class="popover-tab" role="tab" data-popover-tab="all" aria-selected="true">All</button>
      ${providerTabsHtml}
    </div>
  `;
}

// Top bar: just the segmented view control — no app name / [update all] row
// above it, so the tabs sit at the very top of the panel. [update all] moved
// into the footer's icon-button row (see buildPopoverFooterHtml); [info]/
// [gear] are still navigation *out* of the panel, the way system panels put
// their "… Settings" row at the bottom. See docs/desktop/mac-popover.md#toolbar.
export function buildPopoverToolbarHtml(providerIds) {
  return `
    <div class="popover-topbar">
      ${buildPopoverTabsHtml(providerIds)}
    </div>
  `;
}

// Footer: a hairline, then the "AI Limits" menu row and three icon buttons —
// update, info, settings — at its right edge.
export function buildPopoverFooterHtml() {
  return `
    <div class="popover-footer">
      <button type="button" class="popover-menu-item" data-popover-open-app>AI&nbsp;Limits</button>
      <button type="button" class="popover-icon-button" data-popover-update-all aria-label="Update all">${POPOVER_UPDATE_ALL_ICON_SVG}</button>
      <button type="button" class="popover-icon-button" data-popover-info aria-label="Help">${POPOVER_INFO_ICON_SVG}</button>
      <button type="button" class="popover-icon-button" data-popover-gear aria-label="Settings">${POPOVER_GEAR_ICON_SVG}</button>
    </div>
  `;
}

// Filters the provider cards under `root` to the selected tab. "all" is the
// default view the Popover opens on.
export function applyPopoverTabFilter(root, selectedTab) {
  for (const tabButton of root.querySelectorAll("[data-popover-tab]")) {
    tabButton.setAttribute("aria-selected", String(tabButton.dataset.popoverTab === selectedTab));
  }

  for (const block of root.querySelectorAll(".provider-block")) {
    block.hidden = selectedTab !== "all" && block.dataset.providerId !== selectedTab;
  }
}

// Wires the top bar's tab switching. Safe to call again after the toolbar
// markup is rebuilt (settings change) — every element it binds to is part of
// that rebuilt markup, so no listener outlives it.
export function attachPopoverToolbarHandlers(root) {
  for (const tabButton of root.querySelectorAll("[data-popover-tab]")) {
    tabButton.addEventListener("click", () => {
      applyPopoverTabFilter(root, tabButton.dataset.popoverTab);
    });
  }
}

// Marks the card area while there is still content below the fold, so the
// panel can fade its bottom edge instead of letting the footer hairline slice
// a row in half (see `.popover-scroll.has-more-below` in styles/popover.css).
// Lives here rather than in popover.js because the showcase preview needs the
// same behavior and shares this module.
export function observePopoverScroll(root) {
  const scrollArea = root.querySelector(".popover-scroll");
  if (!scrollArea) {
    return;
  }

  const sync = () => {
    const remaining = scrollArea.scrollHeight - scrollArea.scrollTop - scrollArea.clientHeight;
    scrollArea.classList.toggle("has-more-below", remaining > 1);
  };

  scrollArea.addEventListener("scroll", sync, { passive: true });

  // The card list changes height constantly (data resolving, tab filtering,
  // display toggles), and the panel itself is resized to its content, so both
  // ends of the comparison move.
  if (typeof window.ResizeObserver === "function") {
    const observer = new ResizeObserver(sync);
    observer.observe(scrollArea);
    for (const child of scrollArea.children) {
      observer.observe(child);
    }
  }

  sync();
}

// Wires the footer's four actions. Separate from the toolbar because the
// footer is mounted once and never rebuilt — calling this on every toolbar
// rebuild would stack duplicate listeners. Callbacks are optional so callers
// without real behavior (the showcase preview) get no-ops.
export function attachPopoverFooterHandlers(root, { onOpenApp, onUpdateAll, onInfo, onGear } = {}) {
  root.querySelector("[data-popover-open-app]")?.addEventListener("click", () => onOpenApp?.());
  root.querySelector("[data-popover-update-all]")?.addEventListener("click", () => onUpdateAll?.());
  root.querySelector("[data-popover-info]")?.addEventListener("click", () => onInfo?.());
  root.querySelector("[data-popover-gear]")?.addEventListener("click", () => onGear?.());
}
