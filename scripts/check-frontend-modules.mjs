import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, extname, join, normalize, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const projectRoot = resolve(import.meta.dirname, "..");
const modulesRoot = join(projectRoot, "frontend", "modules");

function collectJavaScriptFiles(directory) {
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry);
    return statSync(path).isDirectory() ? collectJavaScriptFiles(path) : extname(path) === ".js" ? [path] : [];
  });
}

const files = collectJavaScriptFiles(modulesRoot);
const importPattern = /(?:import|export)\s+(?:[\s\S]*?\s+from\s+)?["']([^"']+)["']/g;

for (const file of files) {
  const syntax = spawnSync(process.execPath, ["--check", file], { encoding: "utf8" });
  if (syntax.status !== 0) {
    process.stderr.write(syntax.stderr || syntax.stdout);
    process.exit(syntax.status ?? 1);
  }

  const source = readFileSync(file, "utf8");
  for (const match of source.matchAll(importPattern)) {
    const specifier = match[1];
    if (!specifier.startsWith(".")) continue;
    const resolved = normalize(resolve(dirname(file), specifier));
    const pathFromModules = relative(modulesRoot, resolved);
    if (pathFromModules.startsWith("..") || !existsSync(resolved)) {
      console.error(`Missing frontend module import: ${relative(projectRoot, file)} -> ${specifier}`);
      process.exit(1);
    }
  }
}

console.log(`Checked ${files.length} frontend ES modules.`);
