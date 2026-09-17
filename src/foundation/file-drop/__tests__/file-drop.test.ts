import { describe, expect, it } from "vitest";

import { normalizeDragPosition } from "../file-drop";

describe("normalizeDragPosition", () => {
  it("preserves AppKit logical webview coordinates on macOS", () => {
    expect(normalizeDragPosition({ x: 1320, y: 610 }, "macos", 2)).toEqual({ x: 1320, y: 610 });
  });

  it("converts WebView2 physical client pixels to CSS pixels on Windows", () => {
    expect(normalizeDragPosition({ x: 1320, y: 610 }, "windows", 2)).toEqual({ x: 660, y: 305 });
  });
});
