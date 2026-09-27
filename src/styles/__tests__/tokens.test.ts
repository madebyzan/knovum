import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { extname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

import { compile } from "tailwindcss";
import { describe, expect, it } from "vitest";

const sourceRoot = fileURLToPath(new URL("../../", import.meta.url));
const source = readFileSync(new URL("../tokens.css", import.meta.url), "utf8");
const theme = source.match(/@theme static \{([\s\S]*?)^\}/mu)?.[1];
if (!theme) throw new Error("tokens.css must declare its public API in @theme static");
const declarations = [...theme.matchAll(/^\s*(--[a-z0-9*-]+):\s*([^;]+);/gmu)];
const publicNames = declarations.map((match) => match[1]).filter((name) => name !== "--color-*");

async function build(candidates: string[]) {
  const require = createRequire(import.meta.url);
  const defaults = readFileSync(require.resolve("tailwindcss/theme.css"), "utf8");
  const compiler = await compile(
    `@layer theme, base, components, utilities;\n@layer theme {\n${defaults}\n}\n${source}\n@tailwind utilities;`,
  );
  return compiler.build(candidates);
}

function productionSources(directory: string): { path: string; source: string }[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return entry.name === "__tests__" ? [] : productionSources(path);
    if (!entry.isFile() || entry.name.endsWith(".test.ts") || ![".ts", ".vue", ".css"].includes(extname(path))) {
      return [];
    }
    return [{ path, source: readFileSync(path, "utf8") }];
  });
}

describe("design token contract", () => {
  it("defines public values once without a parallel branded namespace", () => {
    expect(publicNames.length).toBeGreaterThan(0);
    expect(new Set(publicNames).size).toBe(publicNames.length);
    expect(theme).not.toContain("--knovum-");
    expect(source).not.toContain("@theme inline");
    for (const [, name, value] of declarations) {
      expect(value.trim(), name).not.toMatch(/^var\(/u);
    }
  });

  it("does not introduce private aliases or token fallback values", () => {
    expect(source).not.toMatch(/--[a-z0-9-]+:\s*var\(/u);
    expect(source).not.toMatch(/var\(\s*--[a-z0-9-]+\s*,/u);
  });

  it("pairs every semantic type size with its line height", () => {
    for (const name of publicNames.filter((item) => /^--text-[a-z]+(?:-[a-z]+)*$/u.test(item))) {
      expect(publicNames, name).toContain(`${name}--line-height`);
    }
  });

  it("emits every public token even without a matching utility consumer", async () => {
    const css = await build([]);
    for (const name of publicNames) expect(css, name).toContain(`${name}:`);
    expect(css).not.toContain("--knovum-color-");
    expect(css).not.toContain("--knovum-font-size-");
  });

  it("generates utilities that read the same runtime variables", async () => {
    const css = await build([
      "bg-canvas",
      "text-body",
      "text-foreground",
      "rounded-control",
      "shadow-canvas-object",
      "ease-standard",
    ]);
    for (const declaration of [
      "background-color: var(--color-canvas)",
      "font-size: var(--text-body)",
      "color: var(--color-foreground)",
      "border-radius: var(--radius-control)",
      "var(--shadow-canvas-object)",
      "transition-timing-function: var(--ease-standard)",
    ]) {
      expect(css).toContain(declaration);
    }
  });

  it("keeps composite shadows live without duplicate utility implementations", async () => {
    const declared = new Set(
      [...source.matchAll(/^\s*--shadow-([a-z0-9-]+):/gmu)].map((match) => match[1]),
    );
    const names = [...declared].filter((name) => !publicNames.includes(`--shadow-${name}`));
    expect(names.length).toBeGreaterThan(0);
    const css = await build(names.map((name) => `shadow-${name}`));
    for (const name of names) {
      expect(theme).not.toContain(`--shadow-${name}:`);
      expect(css).toContain(`--tw-shadow: var(--shadow-${name})`);
      expect(css.split(`.shadow-${name} {`)).toHaveLength(2);
    }
  });

  it("does not put a mode-dependent composite in Tailwind's compile-time shadow scale", () => {
    for (const name of publicNames.filter((item) => item.startsWith("--shadow-"))) {
      const definitions = [...source.matchAll(new RegExp(`^\\s*${name}:`, "gmu"))];
      expect(definitions, name).toHaveLength(1);
    }
  });

  it("keeps mode selectors at equal specificity", () => {
    expect(source).not.toMatch(/:root\[/u);
    expect(source).toContain(':root:where([data-theme="dark"][data-window-active="false"])');
    expect(source).toContain("@media (prefers-contrast: more)");
    expect(source).toContain("@media (prefers-reduced-transparency: reduce)");
  });

  it("resolves every literal private-token reference in production source", () => {
    const files = productionSources(sourceRoot);
    const declared = new Set<string>();
    for (const file of files) {
      for (const match of file.source.matchAll(/(--knovum-[a-z0-9-]+)["']?\s*:/gu)) {
        declared.add(match[1]);
      }
      for (const match of file.source.matchAll(/\.setProperty\(["'](--knovum-[a-z0-9-]+)["']/gu)) {
        declared.add(match[1]);
      }
    }
    const missing = files.flatMap((file) => {
      const references = new Set(
        [...file.source.matchAll(/--knovum-[a-z0-9]+(?:-[a-z0-9]+)*(?![a-z0-9-])/gu)].map((match) => match[0]),
      );
      return [...references]
        .filter((name) => !declared.has(name))
        .map((name) => `${relative(sourceRoot, file.path)}: ${name}`);
    });
    expect(missing).toEqual([]);
  });
});
