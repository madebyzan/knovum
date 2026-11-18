import { describe, expect, it } from "vitest";

import { resolveMinimapVisibility } from "../minimap/minimap-visibility";

const HOST_DESKTOP = { width: 1200, height: 800 };

describe("minimap visibility policy", () => {
  it("hides completely when canvas has no objects, no visible content, or host is too small", () => {
    expect(
      resolveMinimapVisibility({
        open: true,
        navigationEnabled: false,
        hasContent: true,
        hostSize: HOST_DESKTOP,
        recordingDockOpen: false,
      }),
    ).toEqual({ showToggle: false, showPanel: false, panelFitsHost: false });

    expect(
      resolveMinimapVisibility({
        open: true,
        navigationEnabled: true,
        hasContent: false,
        hostSize: HOST_DESKTOP,
        recordingDockOpen: false,
      }).showToggle,
    ).toBe(false);

    expect(
      resolveMinimapVisibility({
        open: true,
        navigationEnabled: true,
        hasContent: true,
        hostSize: { width: 320, height: 220 },
        recordingDockOpen: false,
      }).showToggle,
    ).toBe(false);
  });

  it("remains collapsed when open is false and expands when open is true", () => {
    const collapsed = resolveMinimapVisibility({
      open: false,
      navigationEnabled: true,
      hasContent: true,
      hostSize: HOST_DESKTOP,
      recordingDockOpen: false,
    });
    expect(collapsed).toEqual({ showToggle: true, showPanel: false, panelFitsHost: true });

    const expanded = resolveMinimapVisibility({
      open: true,
      navigationEnabled: true,
      hasContent: true,
      hostSize: HOST_DESKTOP,
      recordingDockOpen: false,
    });
    expect(expanded).toEqual({ showToggle: true, showPanel: true, panelFitsHost: true });
  });

  it("suppresses panel when host is too narrow or recording dock would collide", () => {
    expect(
      resolveMinimapVisibility({
        open: true,
        navigationEnabled: true,
        hasContent: true,
        hostSize: { width: 420, height: 600 },
        recordingDockOpen: false,
      }),
    ).toEqual({ showToggle: true, showPanel: false, panelFitsHost: false });

    expect(
      resolveMinimapVisibility({
        open: true,
        navigationEnabled: true,
        hasContent: true,
        hostSize: { width: 640, height: 700 },
        recordingDockOpen: true,
      }),
    ).toEqual({ showToggle: true, showPanel: false, panelFitsHost: false });
  });
});
