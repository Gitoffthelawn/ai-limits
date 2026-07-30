export const updateFrequencyOptions = [
  "Manual only",
  "1 min",
  "5 min",
  "10 min",
  "30 min",
  "1 hour",
];

export const DEFAULT_UPDATE_FREQUENCY = "5 min";
export const PROVIDER_STATUS_HIDE_MS = 4000;
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

export const SOURCE_PRIORITY_OPTIONS = ["fast", "full", "best"];
export const SOURCE_PRIORITY_LABELS = {
  fast: "Fast",
  full: "Full",
  best: "Best",
};

export const DEFAULT_APP_SETTINGS = {
  notifications: true,
  cursor: true,
  cloud: true,
  codex: true,
  sourcePriority: "full",
};

export const THEME_ACCENTS = {
  danger: [255, 42, 34],
  warning: [255, 207, 22],
  success: [126, 217, 65],
};
