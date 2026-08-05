import { escapeHtml } from "./provider-formatters.js";
import { DEFAULT_HELP_CHAPTER, HELP_CHAPTERS } from "./help-chapters.js";
import { openExternalSetup } from "./links.js";

let helpMenu = null;
let helpContent = null;
let helpView = null;
let homeView = null;
let closeSettingsMenu = null;

export function initHelp(elements, { onCloseSettings }) {
  helpMenu = elements.helpMenu;
  helpContent = elements.helpContent;
  helpView = elements.helpView;
  homeView = elements.homeView;
  closeSettingsMenu = onCloseSettings;
}

function findHelpChapter(chapterId) {
  return HELP_CHAPTERS.find((chapter) => chapter.id === chapterId) ?? HELP_CHAPTERS[0];
}

export function renderHelpMenu() {
  helpMenu.innerHTML = HELP_CHAPTERS.map(
    (chapter) =>
      `<button type="button" class="help-menu-item" data-help-chapter="${chapter.id}">${escapeHtml(chapter.label)}</button>`,
  ).join("");

  for (const button of helpMenu.querySelectorAll("[data-help-chapter]")) {
    button.addEventListener("click", () => {
      selectHelpChapter(button.dataset.helpChapter);
    });
  }
}

function selectHelpChapter(chapterId) {
  const chapter = findHelpChapter(chapterId);

  for (const button of helpMenu.querySelectorAll("[data-help-chapter]")) {
    const isCurrent = button.dataset.helpChapter === chapter.id;
    button.setAttribute("aria-current", isCurrent ? "page" : "false");
  }

  helpContent.innerHTML = chapter.render();
  for (const button of helpContent.querySelectorAll("[data-open-external]")) {
    button.addEventListener("click", () => {
      openExternalSetup(button.dataset.openExternal);
    });
  }
  for (const button of helpContent.querySelectorAll("[data-open-help]")) {
    button.addEventListener("click", () => {
      selectHelpChapter(button.dataset.openHelp);
    });
  }
  for (const button of helpContent.querySelectorAll("[data-copy-text]")) {
    button.addEventListener("click", () => {
      copyHelpText(button);
    });
  }
  const cliCommand = helpContent.querySelector("[data-cli-command]");
  if (cliCommand) {
    loadCliCommand(cliCommand);
  }
  const appVersion = helpContent.querySelector("[data-app-version]");
  if (appVersion) {
    loadAppVersion(appVersion);
  }
  helpContent.querySelector("[data-copy-cli-command]")?.addEventListener("click", (event) => {
    copyHelpText(event.currentTarget);
  });
  helpContent.querySelector("[data-run-cli-command]")?.addEventListener("click", (event) => {
    runCliCommand(event.currentTarget);
  });
}

async function loadCliCommand(commandElement) {
  const commandRow = commandElement.closest(".cli-command-row");
  const copyButton = commandRow.querySelector("[data-copy-cli-command]");
  const runButton = commandRow.querySelector("[data-run-cli-command]");
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    commandElement.textContent = "Available in the desktop app";
    return;
  }
  try {
    const command = await invoke("get_cli_command");
    if (!commandElement.isConnected) {
      return;
    }
    commandElement.textContent = command;
    copyButton.dataset.copyText = command;
    copyButton.disabled = false;
    runButton.disabled = false;
  } catch {
    commandElement.textContent = "Command unavailable";
  }
}

async function loadAppVersion(versionElement) {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return;
  }
  try {
    const version = await invoke("get_app_version");
    if (!versionElement.isConnected) {
      return;
    }
    versionElement.textContent = `Version ${version}`;
  } catch {
    // Leave the element empty if the version can't be read.
  }
}

async function copyHelpText(button) {
  const text = button.dataset.copyText;
  const originalLabel = button.textContent;
  try {
    await navigator.clipboard.writeText(text);
    button.textContent = "Copied";
  } catch {
    button.textContent = "Copy failed";
  }
  setTimeout(() => {
    button.textContent = originalLabel;
  }, 1500);
}

async function runCliCommand(button) {
  const originalLabel = button.textContent;
  try {
    await window.__TAURI__.core.invoke("run_cli_in_terminal");
    button.textContent = "Started";
  } catch {
    button.textContent = "Run failed";
  }
  setTimeout(() => {
    button.textContent = originalLabel;
  }, 1500);
}

export function isHelpOpen() {
  return Boolean(helpView) && !helpView.hidden;
}

// The Popover has no Help view of its own (see Roles in
// docs/desktop/mac-popover.md) and never calls initHelp, so helpView/homeView
// stay null there. Provider cards can still trigger openHelp (e.g. the
// "More details" link on a data-errors state, shared via providers.js) —
// when there's no local Help view to switch to, defer to whatever bridge the
// native layer has wired up to open it in the Main Window instead. See
// window.__openMainWindowHelp in popover.js.
export function openHelp(chapterId = DEFAULT_HELP_CHAPTER) {
  if (!helpView || !homeView) {
    window.__openMainWindowHelp?.(chapterId);
    return;
  }

  closeSettingsMenu?.();
  selectHelpChapter(chapterId);
  homeView.hidden = true;
  helpView.hidden = false;
}

export function closeHelp() {
  helpView.hidden = true;
  homeView.hidden = false;
}
