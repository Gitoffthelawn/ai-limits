#!/usr/bin/env node
import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const outputDir = join(projectRoot, "docs", "readmes", "screenshots");
const host = process.env.AI_LIMITS_TAURI_DEV_HOST || "127.0.0.1";
const port = process.env.AI_LIMITS_TAURI_DEV_PORT || "1420";
const baseUrl = `http://${host}:${port}`;

const shots = [
  { file: "macos.png", platform: "macos", theme: "dark" },
  { file: "windows.png", platform: "windows", theme: "dark" },
  { file: "linux.png", platform: "linux", theme: "dark" },
  { file: "macos-light-settings.png", platform: "macos", theme: "light", openSettings: true },
  { file: "macos-help.png", platform: "macos", theme: "dark", openHelp: true },
];

async function prepareWindow(page, { openSettings, openHelp }) {
  const windowFrame = page.locator(".showcase-window");
  await windowFrame.waitFor({ state: "visible" });
  await page.evaluate(() => {
    document.body.style.background = "transparent";
    document.body.style.backgroundImage = "none";
  });
  await windowFrame.evaluate((el) => {
    el.style.width = "1010px";
    el.style.boxShadow = "none";
  });
  await page.locator(".provider-block").first().waitFor({ state: "visible" });

  if (openSettings) {
    await page.locator("#settings-button").click();
    await page.locator("#settings-dropdown:not([hidden])").waitFor({ state: "visible" });
  }

  if (openHelp) {
    await page.locator("#help-button").click();
    await page.locator("#help-view:not([hidden])").waitFor({ state: "visible" });
  }
}

async function main() {
  await mkdir(outputDir, { recursive: true });
  const browser = await chromium.launch();
  try {
    for (const shot of shots) {
      const context = await browser.newContext({
        viewport: { width: 1400, height: 900 },
        deviceScaleFactor: 1,
      });
      const page = await context.newPage();
      await page.addInitScript(({ themeValue }) => {
        localStorage.setItem("ai-limits-theme", JSON.stringify({ mode: "manual", value: themeValue }));
        localStorage.removeItem("ai-limits-settings");
        localStorage.removeItem("ai-limits-provider-intervals");
      }, { themeValue: shot.theme });
      await page.goto(`${baseUrl}/?showcase=${shot.platform}`, { waitUntil: "networkidle" });
      await prepareWindow(page, shot);
      await page.locator(".showcase-window").screenshot({
        path: join(outputDir, shot.file),
        type: "png",
        omitBackground: true,
      });
      await context.close();
      console.log(`Wrote ${shot.file}`);
    }
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
