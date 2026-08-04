export const updateFrequencyOptions = [
  "Manual only",
  "1 min",
  "5 min",
  "10 min",
  "30 min",
  "1 hour",
];

export const DEFAULT_UPDATE_FREQUENCY = "5 min";
export const SETTINGS_STORAGE_KEY = "ai-limits-settings";
export const PROVIDER_INTERVALS_STORAGE_KEY = "ai-limits-provider-intervals";
export const THEME_STORAGE_KEY = "ai-limits-theme";
export const PROVIDER_IDS = ["codex", "claude", "cursor"];

export const EXTERNAL_LINKS = {
  claude: "https://code.claude.com/docs/en/setup",
  codex: "https://developers.openai.com/codex/cli",
  github: "https://github.com/md2it/ai-limits",
  license: "https://github.com/md2it/ai-limits/blob/main/LICENSE",
};

export const DEFAULT_APP_SETTINGS = {
  notifications: true,
  autoUpdate: true,
  cursor: true,
  cloud: true,
  codex: true,
  showLimits: true,
  showPlan: true,
  showSource: false,
  showUpdateTime: true,
};

/// How long the app waits between automatic update checks while it stays open.
/// A check also runs once at startup.
export const APP_UPDATE_CHECK_INTERVAL_MS = 12 * 60 * 60 * 1000;
