import { describe, expect, it } from "vitest";

import { flattenMenuGroups, menuItem, normalizeAccelerator } from "../popup";

describe("context-menu", () => {
  it("normalizes $mod accelerators to CmdOrCtrl", () => {
    expect(normalizeAccelerator("$mod+c")).toBe("CmdOrCtrl+C");
    expect(normalizeAccelerator("$mod+shift+p")).toBe("CmdOrCtrl+Shift+P");
    expect(normalizeAccelerator("alt+delete")).toBe("Alt+Delete");
    expect(normalizeAccelerator(undefined)).toBeUndefined();
  });

  it("creates menu items with normalized accelerators", () => {
    const item = menuItem("Copy", () => {}, { accelerator: "$mod+c" });
    expect(item.text).toBe("Copy");
    expect(item.accelerator).toBe("CmdOrCtrl+C");
    expect(item.enabled).toBe(true);
  });

  it("flattens 2D groups and inserts separators", () => {
    const groups = [[menuItem("Cut", () => {}), menuItem("Copy", () => {})], [menuItem("Delete", () => {})]];
    const flattened = flattenMenuGroups(groups);
    expect(flattened).toHaveLength(4);
    expect((flattened[0] as { text: string }).text).toBe("Cut");
    expect((flattened[1] as { text: string }).text).toBe("Copy");
    expect((flattened[2] as { item: string }).item).toBe("Separator");
    expect((flattened[3] as { text: string }).text).toBe("Delete");
  });
});
