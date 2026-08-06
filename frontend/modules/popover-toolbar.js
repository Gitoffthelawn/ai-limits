// Shared chrome (the single header row) for the macOS Menu Bar Popover, per
// docs/desktop/mac-popover.md#toolbar.
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

function buildPopoverIconSvg(paths) {
  return `
    <svg class="popover-icon" xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>
  `;
}

// A single circular refresh arrow — the system-standard "refresh everything"
// shape, drawn directly instead of reshaping another module's markup.
const POPOVER_UPDATE_ALL_ICON_SVG = buildPopoverIconSvg(`
    <path d="M20.5 12a8.5 8.5 0 1 1-2.49-6.01" />
    <path d="M20.5 4.5V10h-5.5" />
`);

// Lucide "info": a circle with a lower stem and a dot — the same glyph the
// Main Window's Help entry point carries, redrawn at popover weight.
const POPOVER_INFO_ICON_SVG = buildPopoverIconSvg(`
    <circle cx="12" cy="12" r="9" />
    <path d="M12 16.5v-5" />
    <path d="M12 8h.01" />
`);

// Lucide "settings" gear, likewise redrawn at popover weight.
const POPOVER_GEAR_ICON_SVG = buildPopoverIconSvg(`
    <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
    <circle cx="12" cy="12" r="3" />
`);

// Header: the "AI Limits" entry point into the Main Window on the left, then
// the icon buttons — update, info, settings — at its right edge, with a
// hairline separating it from the scrollable card list below. Used to sit
// split across a top bar (View Tabs) and a footer (this row); now that View
// Tabs are gone the whole thing lives in one header row instead.
export function buildPopoverHeaderHtml() {
  return `
    <div class="popover-header">
      <button type="button" class="popover-menu-item" data-popover-open-app>AI&nbsp;Limits</button>
      <button type="button" class="popover-icon-button" data-popover-update-all aria-label="Update all">${POPOVER_UPDATE_ALL_ICON_SVG}</button>
      <button type="button" class="popover-icon-button" data-popover-info aria-label="Help">${POPOVER_INFO_ICON_SVG}</button>
      <button type="button" class="popover-icon-button" data-popover-gear aria-label="Settings">${POPOVER_GEAR_ICON_SVG}</button>
    </div>
  `;
}

// Marks the card area while there is still content below the fold, so the
// panel can fade its bottom edge instead of clipping a row abruptly. The
// class is toggled from JS and removed at the bottom of the list, so a panel
// whose content fits is never dimmed. Lives here rather than in popover.js
// because the showcase preview needs the same behavior and shares this
// module.
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

  // The card list changes height constantly (data resolving, display
  // toggles), and the panel itself is resized to its content, so both ends
  // of the comparison move.
  if (typeof window.ResizeObserver === "function") {
    const observer = new ResizeObserver(sync);
    observer.observe(scrollArea);
    for (const child of scrollArea.children) {
      observer.observe(child);
    }
  }

  sync();
}

// Wires the header's four actions. Mounted once and never rebuilt, so this
// is called once too. Callbacks are optional so callers without real
// behavior (the showcase preview) get no-ops.
export function attachPopoverHeaderHandlers(root, { onOpenApp, onUpdateAll, onInfo, onGear } = {}) {
  root.querySelector("[data-popover-open-app]")?.addEventListener("click", () => onOpenApp?.());
  root.querySelector("[data-popover-update-all]")?.addEventListener("click", () => onUpdateAll?.());
  root.querySelector("[data-popover-info]")?.addEventListener("click", () => onInfo?.());
  root.querySelector("[data-popover-gear]")?.addEventListener("click", () => onGear?.());
}
