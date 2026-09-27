import { readFileSync } from "node:fs";

import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { defineComponent, nextTick } from "vue";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useCanvasPalette } from "../canvas/konva/presentation/use-canvas-palette";

const tokens = readFileSync(new URL("../../../styles/tokens.css", import.meta.url), "utf8");
const colorNames = new Set(
  [...tokens.matchAll(/(--(?:color-[a-z0-9-]+|knovum-canvas-(?:grid|measurement)-color)):/gu)].map(
    (match) => match[1],
  ),
);
const wrappers: VueWrapper[] = [];
const media = new Map<string, EventTarget>();
let originalStyle: string | null;
let originalTheme: string | undefined;
let originalActivity: string | undefined;

function mountPalette() {
  let palette: ReturnType<typeof useCanvasPalette> | undefined;
  const wrapper = mount(
    defineComponent({
      setup() {
        palette = useCanvasPalette();
        return () => null;
      },
    }),
  );
  wrappers.push(wrapper);
  if (!palette) throw new Error("Canvas palette was not initialized during setup");
  return { palette, wrapper };
}

beforeEach(() => {
  const root = document.documentElement;
  originalStyle = root.getAttribute("style");
  originalTheme = root.dataset.theme;
  originalActivity = root.dataset.windowActive;
  for (const name of colorNames) root.style.setProperty(name, "rgb(10 20 30)");
  delete root.dataset.theme;
  delete root.dataset.windowActive;
  vi.stubGlobal("matchMedia", (query: string) => {
    const target = new EventTarget();
    media.set(query, target);
    return Object.assign(target, { media: query, matches: false });
  });
});

afterEach(() => {
  for (const wrapper of wrappers.splice(0)) wrapper.unmount();
  const root = document.documentElement;
  if (originalStyle === null) root.removeAttribute("style");
  else root.setAttribute("style", originalStyle);
  if (originalTheme === undefined) delete root.dataset.theme;
  else root.dataset.theme = originalTheme;
  if (originalActivity === undefined) delete root.dataset.windowActive;
  else root.dataset.windowActive = originalActivity;
  media.clear();
  vi.unstubAllGlobals();
});

describe("canvas palette", () => {
  it("reads the CSS source synchronously instead of publishing a hard-coded initial theme", () => {
    document.documentElement.style.setProperty("--color-canvas-note-yellow", "rgb(40 50 60)");
    const { palette } = mountPalette();
    expect(palette.value.accent).toBe("rgb(10 20 30)");
    expect(palette.value.noteFill.yellow).toBe("rgb(40 50 60)");
    expect(palette.value.relation.neutral).toBe(palette.value.foregroundSecondary);
  });

  it("fails when a required token is missing", () => {
    document.documentElement.style.removeProperty("--color-canvas-note-yellow");
    expect(mountPalette).toThrow("Canvas color token --color-canvas-note-yellow is not defined");
  });

  it.each(["theme", "windowActive"])("refreshes when the %s mode changes", async (attribute) => {
    const { palette } = mountPalette();
    await nextTick();
    document.documentElement.style.setProperty("--color-foreground-secondary", "rgb(40 50 60)");
    document.documentElement.dataset[attribute] = attribute === "theme" ? "dark" : "false";
    await flushPromises();
    expect(palette.value.foregroundSecondary).toBe("rgb(40 50 60)");
    expect(palette.value.relation.neutral).toBe("rgb(40 50 60)");
  });

  it.each(["(prefers-contrast: more)", "(prefers-reduced-transparency: reduce)"])(
    "refreshes when %s changes without a theme attribute mutation",
    async (query) => {
      const { palette } = mountPalette();
      await nextTick();
      document.documentElement.style.setProperty("--color-border", "rgb(40 50 60)");
      const target = media.get(query);
      if (!target) throw new Error(`No media listener for ${query}`);
      target.dispatchEvent(new Event("change"));
      expect(palette.value.border).toBe("rgb(40 50 60)");
    },
  );

  it("stops attribute and media listeners when the canvas unmounts", async () => {
    const { palette, wrapper } = mountPalette();
    await nextTick();
    wrapper.unmount();
    wrappers.splice(wrappers.indexOf(wrapper), 1);
    const snapshot = palette.value;
    document.documentElement.style.setProperty("--color-border", "rgb(40 50 60)");
    document.documentElement.dataset.theme = "dark";
    for (const target of media.values()) target.dispatchEvent(new Event("change"));
    await flushPromises();
    expect(palette.value).toBe(snapshot);
  });
});
