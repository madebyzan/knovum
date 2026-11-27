// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";

import { openMermaidLightbox } from "../mermaid-lightbox";
import { createMermaidStage } from "../mermaid-stage";
import {
  computeInlineMermaidScale,
  computeLightboxFitScale,
  MERMAID_MIN_INLINE_SCALE,
  parseSvgDimensions,
  zoomAroundAnchor,
} from "../mermaid-viewport";

const WIDE_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1400 600" width="1400" height="600"><rect width="1400" height="600"/></svg>`;
const TALL_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 960" width="360" height="960"><rect width="360" height="960"/></svg>`;
const COMPACT_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 180" width="320" height="180"><rect width="320" height="180"/></svg>`;

afterEach(() => {
  document.body.replaceChildren();
});

describe("mermaid viewport geometry", () => {
  it("extracts intrinsic dimensions from viewBox before width/height attributes", () => {
    const doc = new DOMParser().parseFromString(WIDE_SVG, "image/svg+xml");
    expect(parseSvgDimensions(doc.documentElement)).toEqual({ width: 1400, height: 600 });
  });

  it("keeps narrow diagrams at 100%, fits medium ones, and enforces a readable floor on wide diagrams", () => {
    expect(computeInlineMermaidScale({ width: 320, height: 180 }, 700)).toBe(1);
    expect(computeInlineMermaidScale({ width: 800, height: 400 }, 700)).toBe(0.88);
    expect(computeInlineMermaidScale({ width: 1400, height: 600 }, 700)).toBe(MERMAID_MIN_INLINE_SCALE);
    expect(computeInlineMermaidScale({ width: 360, height: 960 }, 700)).toBe(1);
  });

  it("frames lightbox fit scale inside viewport and keeps zoom anchor stationary", () => {
    const fit = computeLightboxFitScale({ width: 1400, height: 600 }, { width: 1200, height: 800 });
    expect(fit).toBeCloseTo(0.78, 2);

    const next = zoomAroundAnchor({ scale: 1, x: 0, y: 0 }, 2, { x: 100, y: -50 });
    expect(next).toEqual({ scale: 2, x: -100, y: 50 });
  });
});

describe("mermaid inline stage & unified fullscreen lightbox", () => {
  it("renders a single fullscreen trigger on reading stages and opens/closes the vector lightbox", () => {
    const host = document.createElement("div");
    Object.defineProperty(host, "clientWidth", { value: 720, configurable: true });
    document.body.append(host);

    const stage = createMermaidStage(host);
    expect(stage.update(WIDE_SVG, false)).toBe(true);
    expect(host.querySelector(".ui-mermaid-viewport")?.getAttribute("data-pannable")).toBe("true");
    expect(host.querySelectorAll("button")).toHaveLength(1);

    host.querySelector<HTMLButtonElement>('[data-testid="mermaid-fullscreen"]')!.click();
    const lightbox = document.querySelector<HTMLElement>('[data-testid="mermaid-lightbox"]')!;
    expect(lightbox.dataset.surfaceTheme).toBe("light");
    const resetBtn = lightbox.querySelector<HTMLButtonElement>('[data-testid="mermaid-lightbox-reset"]')!;
    expect(resetBtn.textContent).toBe("100%");

    lightbox.querySelector<HTMLButtonElement>('[data-testid="mermaid-lightbox-zoom-in"]')!.click();
    expect(Number.parseInt(resetBtn.textContent ?? "0", 10)).toBeGreaterThan(100);

    globalThis.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(document.querySelector('[data-testid="mermaid-lightbox"]')).toBeNull();

    stage.destroy();
  });

  it("marks tall narrow diagrams as pannable and omits floating pill when editor header owns expand", () => {
    const host = document.createElement("div");
    Object.defineProperty(host, "clientWidth", { value: 720, configurable: true });
    document.body.append(host);

    const stage = createMermaidStage(host, { showExpandButton: false });
    expect(stage.update(TALL_SVG, false)).toBe(true);
    expect(host.querySelector(".ui-mermaid-viewport")?.getAttribute("data-pannable")).toBe("true");
    expect(host.querySelector(".ui-mermaid-expand-pill")).toBeNull();

    stage.openFullscreen();
    expect(document.querySelector('[data-testid="mermaid-lightbox"]')).not.toBeNull();
    globalThis.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

    stage.destroy();
  });

  it("supports dark fullscreen lightbox on compact diagrams without pan cursor", () => {
    const host = document.createElement("div");
    Object.defineProperty(host, "clientWidth", { value: 720, configurable: true });
    document.body.append(host);

    const stage = createMermaidStage(host);
    expect(stage.update(COMPACT_SVG, true)).toBe(true);
    expect(host.querySelector(".ui-mermaid-viewport")?.getAttribute("data-pannable")).toBe("false");

    const close = openMermaidLightbox({ svg: COMPACT_SVG, dark: true });
    expect(
      document.querySelector<HTMLElement>('[data-testid="mermaid-lightbox"]')?.dataset.surfaceTheme,
    ).toBe("dark");
    close();
    expect(document.querySelector('[data-testid="mermaid-lightbox"]')).toBeNull();

    stage.destroy();
  });
});
