import { describe, expect, it } from "vitest";

import { isScrollableTarget } from "../scrollable-target";

describe("isScrollableTarget", () => {
  it("returns false for null or non-elements", () => {
    expect(isScrollableTarget(null)).toBe(false);
    expect(isScrollableTarget(undefined as never)).toBe(false);
  });

  it("returns false for canvas elements", () => {
    const canvas = document.createElement("canvas");
    expect(isScrollableTarget(canvas)).toBe(false);
  });

  it("returns true for elements with data-canvas-scrollable or scroll overflow classes", () => {
    const container = document.createElement("div");
    container.setAttribute("data-canvas-scrollable", "");
    const child = document.createElement("span");
    container.appendChild(child);

    expect(isScrollableTarget(container)).toBe(true);
    expect(isScrollableTarget(child)).toBe(true);
  });

  it("returns true for elements with overflow utility classes", () => {
    const container = document.createElement("div");
    container.className = "min-h-0 flex-1 overflow-y-auto";
    const inner = document.createElement("p");
    container.appendChild(inner);

    expect(isScrollableTarget(container)).toBe(true);
    expect(isScrollableTarget(inner)).toBe(true);
  });
});
