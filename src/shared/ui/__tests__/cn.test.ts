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

// Token completeness against tokens.css is a source gate: tools/check-cn-token-coverage.mjs. These
// cases pin the mechanism that gate protects, since a font size classified as a colour is dropped
// whenever a real text colour is present.
describe("cn design tokens", () => {
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
