const SHOWCASE_PLATFORMS = ["macos", "windows", "linux"];
const SHOWCASE_PLATFORM_LABELS = { macos: "macOS", windows: "Windows", linux: "Linux" };
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
    dataTimestamp: "20:03",
    selectedUpdateFrequency: "5 min",
    limits: [
      { label: "7d", remainingPercentage: 67.0, resetTime: "Aug 2, 00:18" },
    ],
    creditsRemaining: 39,
    availableLimitResets: 1,
    // Codex reports a plan name but no price and no management links, so the
    // subscription section is two lines and carries no link line.
    plan: {
      lines: ["Plus", "renews Sep 3, 2026"],
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
    dataTimestamp: "20:03",
    selectedUpdateFrequency: "5 min",
    limits: [
      { label: "5h", remainingPercentage: 77.0, resetTime: "20:39" },
      { label: "7d", remainingPercentage: 40.0, resetTime: "Jul 28, 15:00" },
    ],
    creditsRemaining: null,
    availableLimitResets: null,
    // Claude exposes no plan data at all, so the card shows no
    // PLAN heading and no rule where one would have been.
    plan: {
      lines: [],
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
    dataTimestamp: "20:02",
    selectedUpdateFrequency: "5 min",
    limits: [
      { label: "Cursor Models", remainingPercentage: 53.9, resetTime: null },
      { label: "Other Models", remainingPercentage: 0.0, resetTime: null },
    ],
    creditsRemaining: null,
    availableLimitResets: null,
    // The Cursor source carries no tier name and no price, so the
    // subscription section is the renewal line alone, with no links.
    plan: {
      lines: ["renews Jul 28, 2026"],
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
  const windowFrame = createShowcaseWindow();
  const captureArea = document.createElement("div");
  captureArea.className = "showcase-capture-area";

  app.replaceWith(stage);
  stage.append(controls, captureArea);
  captureArea.append(windowFrame);
  windowFrame.append(app);
  attachShowcaseResize(windowFrame, captureArea);
}
