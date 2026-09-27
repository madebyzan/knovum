import { useEventListener, useMutationObserver } from "@vueuse/core";
import { shallowRef } from "vue";

import type { CanvasNodeAppearance, CanvasRelationColor } from "../../schema/document";

type CanvasFillPreset = NonNullable<CanvasNodeAppearance["fill"]>;

export interface CanvasPalette {
  accent: string;
  border: string;
  borderStrong: string;
  canvas: string;
  foreground: string;
  foregroundSecondary: string;
  grid: string;
  measurement: string;
  contentFill: Record<CanvasFillPreset, string>;
  panel: string;
  relation: Record<CanvasRelationColor, string>;
}

export function useCanvasPalette() {
  const palette = shallowRef(readPalette());
  const refresh = () => {
    palette.value = readPalette();
  };

  useMutationObserver(document.documentElement, refresh, {
    attributes: true,
    attributeFilter: ["data-theme", "data-window-active"],
  });
  useEventListener(globalThis.matchMedia("(prefers-contrast: more)"), "change", refresh);
  return palette;
}

function readPalette(): CanvasPalette {
  const styles = getComputedStyle(document.documentElement);
  return {
    accent: value(styles, "--color-stroke-accent"),
    border: value(styles, "--color-stroke-default"),
    borderStrong: value(styles, "--color-stroke-strong"),
    canvas: value(styles, "--color-surface-canvas"),
    foreground: value(styles, "--color-fg-primary"),
    foregroundSecondary: value(styles, "--color-fg-secondary"),
    grid: value(styles, "--knovum-space-grid-color"),
    measurement: value(styles, "--color-content-amber"),
    contentFill: {
      yellow: value(styles, "--color-content-yellow-subtle"),
      green: value(styles, "--color-content-green-subtle"),
      blue: value(styles, "--color-content-blue-subtle"),
      pink: value(styles, "--color-content-pink-subtle"),
      gray: value(styles, "--color-content-gray-subtle"),
    },
    panel: value(styles, "--color-surface-panel"),
    relation: {
      neutral: value(styles, "--color-fg-secondary"),
      blue: value(styles, "--color-content-blue"),
      green: value(styles, "--color-content-green"),
      amber: value(styles, "--color-content-amber"),
      red: value(styles, "--color-content-red"),
      purple: value(styles, "--color-content-purple"),
    },
  };
}

function value(styles: CSSStyleDeclaration, name: string) {
  const resolved = styles.getPropertyValue(name).trim();
  if (!resolved) throw new Error(`Canvas color token ${name} is not defined`);
  return resolved;
}
