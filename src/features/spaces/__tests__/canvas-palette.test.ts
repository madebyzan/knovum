import { readFileSync } from "node:fs";

import { mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, nextTick } from "vue";

import { useCanvasPalette } from "../canvas/konva/presentation/use-canvas-palette";

const source =
  readFileSync("src/styles/tokens.css", "utf8") +
  readFileSync("src/features/spaces/presentation/space-tokens.css", "utf8");
let styles: CSSStyleDeclaration;
let contrast: MediaQueryList;
let cleanup: (() => void) | undefined;

beforeEach(() => {
  styles = document.createElement("div").style;
  for (const [, name] of source.matchAll(/(--(?:knovum|color)-[\w-]+):/g))
    styles.setProperty(name!, "rgb(1, 2, 3)");
  styles.setProperty("--font-sans", "system-ui");
  styles.setProperty("--text-label", "0.8125rem");
  styles.setProperty("--text-caption", "0.75rem");
  styles.fontSize = "16px";
  contrast = globalThis.matchMedia("(prefers-contrast: more)");
  vi.spyOn(globalThis, "matchMedia").mockReturnValue(contrast);
  vi.spyOn(globalThis, "getComputedStyle").mockReturnValue(styles);
});

afterEach(() => {
  cleanup?.();
  cleanup = undefined;
  document.documentElement.removeAttribute("data-theme");
  document.documentElement.removeAttribute("data-window-active");
  document.documentElement.removeAttribute("data-text-size");
  vi.restoreAllMocks();
});

function mountPalette() {
  let palette: ReturnType<typeof useCanvasPalette> | undefined;
  const wrapper = mount(
    defineComponent({
      setup() {
        palette = useCanvasPalette();
        return () => h("div");
      },
    }),
  );
  cleanup = () => wrapper.unmount();
  if (!palette) throw new Error("Canvas palette did not initialize");
  return palette;
}

describe("canvas token projection", () => {
  it("reads the canonical palette before the first render without a fallback", () => {
    styles.setProperty("--color-content-green", "rgb(4, 5, 6)");
    const palette = mountPalette();
    expect(palette.value.relation.green).toBe("rgb(4, 5, 6)");
    expect(palette.value.canvas).toBe("rgb(1, 2, 3)");
    expect(palette.value.typography).toEqual({ fontFamily: "system-ui", labelSize: 13, captionSize: 12 });
  });

  it.each(["data-theme", "data-window-active"])(
    "refreshes canvas paint when %s changes",
    async (attribute) => {
      const palette = mountPalette();
      await nextTick();
      styles.setProperty("--color-fg-secondary", "rgb(4, 5, 6)");
      document.documentElement.setAttribute(attribute, attribute === "data-theme" ? "dark" : "false");
      await vi.waitFor(() => expect(palette.value.foregroundSecondary).toBe("rgb(4, 5, 6)"));
      expect(palette.value.relation.neutral).toBe("rgb(4, 5, 6)");
    },
  );

  it("refreshes container typography when the text-size preference changes", async () => {
    const palette = mountPalette();
    styles.setProperty("--text-label", "0.875rem");
    document.documentElement.setAttribute("data-text-size", "large");
    await vi.waitFor(() => expect(palette.value.typography.labelSize).toBe(14));
  });

  it("refreshes canvas boundaries when the preferred contrast changes", async () => {
    const palette = mountPalette();
    await nextTick();
    styles.setProperty("--color-stroke-default", "rgb(7, 8, 9)");
    contrast.dispatchEvent(new Event("change"));
    expect(palette.value.border).toBe("rgb(7, 8, 9)");
  });

  it("fails visibly when a required token is absent", () => {
    styles.removeProperty("--knovum-space-grid-color");
    vi.spyOn(console, "warn").mockImplementation(() => undefined);
    expect(() => mountPalette()).toThrow("Canvas color token --knovum-space-grid-color is not defined");
  });
});
