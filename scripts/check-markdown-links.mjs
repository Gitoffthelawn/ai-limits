import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

const projectRoot = resolve(import.meta.dirname, "..");
const markdownFiles = [];
const headings = new Map();

function collectMarkdownFiles(directory) {
  for (const entry of readdirSync(directory)) {
    if ([".git", "node_modules", "target", "tmp"].includes(entry)) continue;
    const path = join(directory, entry);
    if (statSync(path).isDirectory()) collectMarkdownFiles(path);
    else if (path.endsWith(".md")) markdownFiles.push(path);
  }
}

function slugify(heading) {
  return heading.toLowerCase().trim().replace(/[^\p{L}\p{N}\s-]/gu, "").replace(/\s+/g, "-");
}

function anchorsFor(path) {
  if (headings.has(path)) return headings.get(path);
  const anchors = new Set();
  const counts = new Map();
  for (const line of readFileSync(path, "utf8").split("\n")) {
    const match = line.match(/^#{1,6}\s+(.+?)\s*#*\s*$/);
    if (!match) continue;
    const base = slugify(match[1]);
    const count = counts.get(base) ?? 0;
    counts.set(base, count + 1);
    anchors.add(count === 0 ? base : `${base}-${count}`);
  }
  headings.set(path, anchors);
  return anchors;
}

function targetParts(target) {
  const clean = target.trim().replace(/^<|>$/g, "").split(/\s+/, 1)[0].split("?", 1)[0];
  const hash = clean.indexOf("#");
  return hash === -1 ? [clean, ""] : [clean.slice(0, hash), decodeURIComponent(clean.slice(hash + 1))];
}

collectMarkdownFiles(projectRoot);
for (const file of markdownFiles) {
  const source = readFileSync(file, "utf8").replace(/```[\s\S]*?```/g, "");
  for (const match of source.matchAll(/!?\[[^\]]*\]\(([^)]+)\)/g)) {
    const [pathPart, anchor] = targetParts(match[1]);
    if (/^(https?:|mailto:|tel:|data:)/i.test(pathPart)) continue;
    const target = pathPart ? resolve(dirname(file), pathPart) : file;
    if (!existsSync(target)) {
      console.error(`Missing Markdown link target: ${relative(projectRoot, file)} -> ${match[1]}`);
      process.exit(1);
    }
    if (anchor && target.endsWith(".md") && !anchorsFor(target).has(anchor)) {
      console.error(`Missing Markdown anchor: ${relative(projectRoot, file)} -> ${match[1]}`);
      process.exit(1);
    }
  }
}

console.log(`Checked Markdown links in ${markdownFiles.length} files.`);
