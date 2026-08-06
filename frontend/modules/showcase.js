import {
  buildPopoverHeaderHtml,
  attachPopoverHeaderHandlers,
  observePopoverScroll,
} from "./popover-toolbar.js";

const SHOWCASE_PLATFORMS = ["macos", "windows", "linux", "popover"];
const SHOWCASE_PLATFORM_LABELS = { macos: "macOS", windows: "Windows", linux: "Linux", popover: "Popover" };
const SHOWCASE_RESIZE_ZONE_PX = 16;
const SHOWCASE_MIN_WIDTH_PX = 320;
const SHOWCASE_MIN_HEIGHT_PX = 540;

const showcasePlatform = new URLSearchParams(window.location.search).get("showcase");
export const isScreenshotShowcase = SHOWCASE_PLATFORMS.includes(showcasePlatform);

export const SHOWCASE_PROVIDERS = {
  codex: {
    id: "codex",
    label: "Codex",
    sourceId: "codex-rpc",
    dataTimestamp: "16:34",
    limits: [
      { label: "7d", remainingPercentage: 23.2, resetTime: "Aug 8, 16:44" },
    ],
    creditsRemaining: 23.8,
    availableLimitResets: 1,
    // Codex reports a plan name but no price and no management links, so the
    // subscription section is a single line and carries no link line.
    plan: {
      lines: ["Plus"],
      links: [],
    },
    errorMessage: null,
    noFreshData: false,
    authorizationRequired: null,
  },
  claude: {
    id: "claude",
    label: "Claude",
    sourceId: "claude-rpc",
    dataTimestamp: "16:41",
    limits: [
      { label: "5h", remainingPercentage: 4.0, resetTime: "21:40" },
      { label: "7d", remainingPercentage: 79.0, resetTime: "Aug 11, 15:00" },
    ],
    creditsRemaining: null,
    availableLimitResets: null,
    // Claude reports a plan name but no price and no management links, so the
    // subscription section is a single line and carries no link line.
    plan: {
      lines: ["Pro"],
      links: [],
    },
    errorMessage: null,
    noFreshData: false,
    authorizationRequired: null,
  },
  cursor: {
    id: "cursor",
    label: "Cursor",
    sourceId: "cursor-api2",
    dataTimestamp: "16:34",
    limits: [
      { label: "Cursor Models", remainingPercentage: 82.7, resetTime: "Aug 28, 05:54" },
      { label: "Other Models", remainingPercentage: 74.8, resetTime: "Aug 28, 05:54" },
    ],
    creditsRemaining: null,
    availableLimitResets: null,
    // The Cursor source carries a tier name with an approximate price, plus
    // a renewal line, and no management links.
    plan: {
      lines: ["Pro ≈ $20.00 /mo", "renews Aug 28, 2026"],
      links: [],
    },
    errorMessage: null,
    noFreshData: false,
    authorizationRequired: null,
  },
};

function createShowcasePlatformControls() {
  const controls = document.createElement("nav");
  controls.className = "showcase-platforms";
  controls.setAttribute("aria-label", "Window appearance");
  controls.innerHTML = SHOWCASE_PLATFORMS.map((platform) => `
    <button type="button" data-showcase-platform="${platform}" aria-pressed="${platform === showcasePlatform}">${SHOWCASE_PLATFORM_LABELS[platform]}</button>
  `).join("");

  for (const button of controls.querySelectorAll("[data-showcase-platform]")) {
    button.addEventListener("click", () => {
      const url = new URL(window.location.href);
      url.searchParams.set("showcase", button.dataset.showcasePlatform);
      window.location.assign(url);
    });
  }

  return controls;
}

function createShowcaseWindow() {
  const windowFrame = document.createElement("section");
  windowFrame.className = `showcase-window showcase-window--${showcasePlatform}`;
  windowFrame.setAttribute("aria-label", `AI Limits on ${SHOWCASE_PLATFORM_LABELS[showcasePlatform]}`);
  windowFrame.innerHTML = `
    <header class="showcase-titlebar">
      <div class="showcase-window-controls" aria-hidden="true"><span></span><span></span><span></span></div>
      <div class="showcase-window-brand"><span class="showcase-app-icon" aria-hidden="true"></span><span class="showcase-window-title">AI&nbsp;Limits</span></div>
      <div class="showcase-titlebar-spacer" aria-hidden="true"></div>
    </header>
  `;
  return windowFrame;
}

// The header markup/behavior lives in popover-toolbar.js, shared with the
// real popover.js surface (frontend/modules/popover.js) so this preview and
// the real Popover window never drift apart.

// The popover is not an OS window: it has no titlebar and is anchored under
// a menu bar icon, not resizable. It reuses the same live `#provider-list`
// already mounted (and fed by SHOWCASE_PROVIDERS) inside `.app` by the
// existing provider pipeline, rather than rendering a second copy — so the
// cards shown here are byte-identical to the ones on the other platforms,
// with their real interactivity intact.
//
// The panel's structure and every `.popover-*` style come from the same two
// files the real window uses (popover-toolbar.js, styles/popover.css), so
// this preview and frontend/popover.html render the same surface. See
// docs/desktop/mac-popover.md.
function createPopoverSurface(app) {
  const providerList = app.querySelector("#provider-list");
  const root = document.createElement("section");
  root.className = "popover-root";
  root.setAttribute("aria-label", "AI Limits menu bar popover");
  root.innerHTML = `
    <div class="popover-mount">${buildPopoverHeaderHtml()}</div>
    <div class="popover-scroll"></div>
  `;
  root.querySelector(".popover-scroll").append(providerList);

  // [update all], [info], [gear] and "Open Application" have no real behavior
  // in the preview: it is a layout surface, not a running app window (the
  // real behavior lives in popover.js).
  attachPopoverHeaderHandlers(root);

  observePopoverScroll(root);

  return root;
}

function attachShowcaseResize(windowFrame, captureArea) {
  windowFrame.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) {
      return;
    }

    const bounds = windowFrame.getBoundingClientRect();
    if (event.clientX < bounds.right - SHOWCASE_RESIZE_ZONE_PX || event.clientY < bounds.bottom - SHOWCASE_RESIZE_ZONE_PX) {
      return;
    }

    event.preventDefault();
    const startX = event.clientX;
    const startY = event.clientY;
    const startWidth = bounds.width;
    const startHeight = bounds.height;
    const captureStyle = window.getComputedStyle(captureArea);
    const captureHorizontalPadding = parseFloat(captureStyle.paddingLeft) + parseFloat(captureStyle.paddingRight);
    const resize = (moveEvent) => {
      const width = Math.max(SHOWCASE_MIN_WIDTH_PX, startWidth + moveEvent.clientX - startX);
      const height = Math.max(SHOWCASE_MIN_HEIGHT_PX, startHeight + moveEvent.clientY - startY);
      windowFrame.style.width = `${width}px`;
      windowFrame.style.height = `${height}px`;
      captureArea.style.width = `${width + captureHorizontalPadding}px`;
      captureArea.style.maxWidth = "none";
    };
    const stopResize = () => {
      window.removeEventListener("pointermove", resize);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
    };

    window.addEventListener("pointermove", resize);
    window.addEventListener("pointerup", stopResize, { once: true });
    window.addEventListener("pointercancel", stopResize, { once: true });
  });
}

export function setupScreenshotShowcase() {
  if (!isScreenshotShowcase) {
    return;
  }

  document.body.classList.add("screenshot-showcase");
  const app = document.querySelector(".app");
  const stage = document.createElement("div");
  stage.className = "showcase-stage";
  const controls = createShowcasePlatformControls();
  const captureArea = document.createElement("div");
  captureArea.className = "showcase-capture-area";

  app.replaceWith(stage);
  stage.append(controls, captureArea);

  if (showcasePlatform === "popover") {
    captureArea.classList.add("showcase-capture-area--popover");
    captureArea.append(createPopoverSurface(app));
    return;
  }

  const windowFrame = createShowcaseWindow();
  captureArea.append(windowFrame);
  windowFrame.append(app);
  attachShowcaseResize(windowFrame, captureArea);
}
