import { describe, expect, it } from "vitest";

import { getProductShortcuts } from "../shortcut-catalog";

describe("shortcut catalog", () => {
  it("provides unique command IDs across all entries", () => {
    const shortcuts = getProductShortcuts();
    const ids = shortcuts.map((item) => item.id);
    const uniqueIds = new Set(ids);
    expect(ids.length).toBe(uniqueIds.size);
  });

  it("covers all primary product categories", () => {
    const shortcuts = getProductShortcuts();
    const categories = new Set(shortcuts.map((item) => item.category));
    expect(categories.has("general")).toBe(true);
    expect(categories.has("navigation")).toBe(true);
    expect(categories.has("canvas")).toBe(true);
    expect(categories.has("editing")).toBe(true);
  });

  it("includes global speech dictation", () => {
    const shortcuts = getProductShortcuts();
    const dictation = shortcuts.find((item) => item.id === "dictation");
    expect(dictation).toBeDefined();
    expect(dictation?.isGlobal).toBe(true);
    expect(dictation?.defaultShortcut).toBe("$mod+Shift+Space");
  });

  it("includes all essential canvas tools and navigation", () => {
    const shortcuts = getProductShortcuts();
    const ids = new Set(shortcuts.map((item) => item.id));
    expect(ids.has("spaces.tool-select")).toBe(true);
    expect(ids.has("spaces.tool-pan")).toBe(true);
    expect(ids.has("spaces.tool-note")).toBe(true);
    expect(ids.has("spaces.tool-text")).toBe(true);
    expect(ids.has("spaces.tool-section")).toBe(true);
    expect(ids.has("spaces.zoom-in")).toBe(true);
    expect(ids.has("spaces.zoom-out")).toBe(true);
    expect(ids.has("spaces.zoom-reset")).toBe(true);
    expect(ids.has("spaces.zoom-to-fit")).toBe(true);
    expect(ids.has("spaces.zoom-to-selection")).toBe(true);
    expect(ids.has("spaces.cancel-canvas-interaction")).toBe(true);
  });

  it("includes all essential structure and editing commands", () => {
    const shortcuts = getProductShortcuts();
    const ids = new Set(shortcuts.map((item) => item.id));
    expect(ids.has("spaces.group-selection")).toBe(true);
    expect(ids.has("spaces.dissolve-group")).toBe(true);
    expect(ids.has("spaces.rename-node")).toBe(true);
    expect(ids.has("spaces.duplicate")).toBe(true);
    expect(ids.has("spaces.delete-selection")).toBe(true);
    expect(ids.has("spaces.undo")).toBe(true);
    expect(ids.has("spaces.redo")).toBe(true);
  });
});
