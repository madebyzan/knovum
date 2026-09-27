import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { cn } from "../cn";

describe("cn", () => {
  it("merges classes cleanly", () => {
    const isHidden = false;
    expect(cn("px-2 py-1", "bg-raised", undefined, isHidden && "hidden")).toBe("px-2 py-1 bg-raised");
  });

  it("resolves conflicting tailwind classes", () => {
    expect(cn("p-2", "p-4")).toBe("p-4");
    expect(cn("text-foreground-secondary", "text-foreground")).toBe("text-foreground");
  });
});

describe("cn design tokens", () => {
  const source = readFileSync("src/styles/tokens.css", "utf8");
  it.each([
    ["text", "text", "text-base"],
    ["radius", "rounded", "rounded-none"],
    ["shadow", "shadow", "shadow-none"],
  ])("merges every %s token in its actual utility group", (namespace, utility, replacement) => {
    const names = [
      ...source.matchAll(new RegExp(`^\\s*--${namespace}-([a-z0-9]+(?:-[a-z0-9]+)*):`, "gm")),
    ].map((match) => match[1]);
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const token = `${utility}-${name}`;
      expect(cn(token, replacement)).toBe(replacement);
      expect(cn(replacement, token)).toBe(token);
    }
  });

  it("keeps every token font size alongside a text colour", () => {
    const names = [...source.matchAll(/^\s*--text-([a-z0-9]+(?:-[a-z0-9]+)*):/gm)];
    expect(names.length).toBeGreaterThan(0);
    for (const [, name] of names) {
      const token = `text-${name}`;
      expect(cn(token, "text-foreground")).toBe(`${token} text-foreground`);
      expect(cn("text-foreground", token)).toBe(`text-foreground ${token}`);
    }
  });

  it("keeps a token font size next to a text colour", () => {
    expect(cn("text-label", "text-foreground")).toBe("text-label text-foreground");
    expect(cn("text-micro text-foreground-tertiary", "text-foreground")).toBe("text-micro text-foreground");
  });

  it("resolves token font sizes against each other", () => {
    expect(cn("text-micro", "text-caption")).toBe("text-caption");
    expect(cn("text-body-lg", "text-label")).toBe("text-label");
  });

  it("resolves token radii against the built-in radius scale", () => {
    expect(cn("rounded-control", "rounded-none")).toBe("rounded-none");
    expect(cn("rounded-none", "rounded-panel")).toBe("rounded-panel");
    expect(cn("rounded-grid-cell", "rounded-section")).toBe("rounded-section");
  });

  it("resolves token shadows against the built-in shadow scale", () => {
    expect(cn("shadow-control", "shadow-none")).toBe("shadow-none");
    expect(cn("shadow-none", "shadow-popover")).toBe("shadow-popover");
  });
});
