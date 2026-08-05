#!/usr/bin/env node
// Rasterizes an SVG file to a transparent PNG at an exact pixel size.
//
// Used by scripts/generate-desktop-icons.sh for the macOS menu bar template
// icon, which the Tauri CLI's own `tauri icon` command cannot produce (it
// only emits the full app-icon set). Renders through the same headless
// Chromium the showcase screenshots already use (scripts/update-showcase-
// screenshots.mjs), so no extra image toolchain is required.
//
// Usage: node scripts/render-svg-png.mjs <input.svg> <output.png> <size>
import { chromium } from "playwright";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const [inputArgument, outputArgument, sizeArgument] = process.argv.slice(2);

if (!inputArgument || !outputArgument || !sizeArgument) {
  process.stderr.write("Usage: node scripts/render-svg-png.mjs <input.svg> <output.png> <size>\n");
  process.exit(1);
}

const size = Number.parseInt(sizeArgument, 10);

if (!Number.isInteger(size) || size <= 0) {
  process.stderr.write(`Invalid size: ${sizeArgument}\n`);
  process.exit(1);
}

const inputPath = resolve(inputArgument);
const outputPath = resolve(outputArgument);
const svg = await readFile(inputPath, "utf8");
const page = `<!doctype html><meta charset="utf-8"><style>
  html, body { margin: 0; padding: 0; background: transparent; }
  svg { display: block; width: ${size}px; height: ${size}px; }
</style>${svg}`;

const browser = await chromium.launch();

try {
  const context = await browser.newContext({
    viewport: { width: size, height: size },
    deviceScaleFactor: 1,
  });
  const tab = await context.newPage();
  await tab.setContent(page, { waitUntil: "load" });
  const png = await tab.screenshot({ omitBackground: true, type: "png" });
  await writeFile(outputPath, png);
} finally {
  await browser.close();
}

process.stdout.write(`Rendered ${outputPath} (${size}x${size})\n`);
