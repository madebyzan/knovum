// @vitest-environment node
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import tailwindcss from "@tailwindcss/vite";
import { createServer } from "vite";
import { beforeAll, describe, expect, it } from "vitest";

const tokenSource = readFileSync("src/styles/tokens.css", "utf8");
const publicTheme = tokenSource.slice(
  tokenSource.indexOf("@theme static"),
  tokenSource.indexOf(":root[data-text-size"),
);
const declarations = new Set([...tokenSource.matchAll(/(--[\w-]+):/g)].map((match) => match[1]));
const productSources = readdirSync("src", { recursive: true, withFileTypes: true })
  .filter((entry) => entry.isFile() && /\.(?:css|ts|vue)$/.test(entry.name))
  .filter((entry) => !entry.parentPath.includes("__tests__") && !entry.name.endsWith(".test.ts"))
  .map((entry) => {
    const path = join(entry.parentPath, entry.name);
    return { path: path.replaceAll("\\", "/"), source: readFileSync(path, "utf8") };
  });

let compiledCss = "";

beforeAll(async () => {
  const server = await createServer({
    configFile: false,
    plugins: [tailwindcss()],
    logLevel: "silent",
    server: { middlewareMode: true, hmr: false, watch: null },
  });
  try {
    const result = await server.transformRequest("/src/styles/app.css?direct");
    if (!result) throw new Error("Design-token stylesheet did not compile");
    compiledCss = result.code;
  } finally {
    await server.close();
  }
}, 30_000);

describe("design-token contract", () => {
  it("resolves every canonical CSS reference and runtime canvas lookup", () => {
    const missing = productSources.flatMap(({ path, source }) =>
      [...source.matchAll(/(?:var\(|["'])(--(?:knovum|color|text)-[\w-]+)/g)]
        .filter((match) => !declarations.has(match[1]!))
        .map((match) => `${path}: ${match[1]}`),
    );
    expect(declarations.size).toBeGreaterThan(0);
    expect(missing).toEqual([]);
  });

  it("emits native theme variables used directly by CSS consumers", () => {
    const references = productSources
      .filter(({ path }) => path !== "src/styles/tokens.css")
      .flatMap(({ source }) =>
        Array.from(source.matchAll(/(?:var\(|["'])(--(?:color|text|font|radius|shadow|ease|blur)-[\w-]+)/g)),
      );
    expect(references.length).toBeGreaterThan(0);
    for (const [, name] of references) expect(compiledCss).toContain(`${name}:`);
  });

  it("keeps feature attributes private and public decisions in native namespaces", () => {
    expect(publicTheme).not.toMatch(/var\(--knovum-space-/);
    expect(tokenSource).not.toContain("@theme inline");
    expect(tokenSource).not.toMatch(/--knovum-(?:color|typography)-/);
    const violations = productSources
      .filter(({ path }) => path !== "src/styles/tokens.css")
      .filter(({ source }) => /var\(--knovum-ref-/.test(source))
      .map(({ path }) => path);
    expect(violations).toEqual([]);
  });

  it("extends Tailwind without duplicating its primitive scales", () => {
    expect(tokenSource).not.toMatch(/--knovum-(?:ref-radius|icon-size|motion-duration)-/);
    expect(tokenSource).not.toContain("--color-*: initial");
    expect(compiledCss).toContain("--radius-md: 0.375rem");
    expect(compiledCss).toContain("--radius-panel: 0.625rem");
    expect(compiledCss).toContain("--radius-section-inner: 0.5625rem");
  });

  it("keeps consumers on the named interaction and warning roles", () => {
    const violations = productSources
      .filter(({ path }) => path !== "src/styles/tokens.css")
      .filter(({ source }) =>
        /hover:bg-surface-\S*-pressed|data-\[state=open\]:bg-surface-\S*-pressed|bg-surface-warning\//.test(
          source,
        ),
      )
      .map(({ path }) => path);
    expect(violations).toEqual([]);
  });

  it.each([
    ["bg-surface-warning-subtle", "background-color", "--color-surface-warning-subtle"],
    ["bg-surface-control-hover", "background-color", "--color-surface-control-hover"],
    ["bg-surface-control-pressed", "background-color", "--color-surface-control-pressed"],
    ["bg-surface-accent", "background-color", "--color-surface-accent"],
    ["text-fg-accent", "color", "--color-fg-accent"],
    ["border-stroke-accent", "border-color", "--color-stroke-accent"],
    ["text-fg-primary", "color", "--color-fg-primary"],
    ["text-body-sm", "font-size", "--text-body-sm"],
    ["text-body-sm", "line-height", "--text-body-sm--line-height"],
  ])("emits %s using the canonical %s role", (utility, property, token) => {
    const rule = compiledCss.match(new RegExp(`\\.${utility}\\s*\\{([^}]+)\\}`));
    expect(rule, `Missing compiled utility ${utility}`).not.toBeNull();
    const value = property === "line-height" ? `var(--tw-leading, var(${token}))` : `var(${token})`;
    expect(rule?.[1]).toContain(`${property}: ${value}`);
  });

  it("emits concrete domain and scrollbar variable utilities without global feature aliases", () => {
    expect(compiledCss).toContain("background-color: var(--knovum-scrollbar-thumb-bg)");
    expect(compiledCss).toContain("border-radius: var(--radius-lg)");
    expect(compiledCss).toContain("--knovum-space-node-box-shadow");
    expect(compiledCss).not.toMatch(/--color-canvas-(?:relation|note|text)-/);
    expect(compiledCss).not.toContain("var(--shadow-toolbar)");
  });
});
