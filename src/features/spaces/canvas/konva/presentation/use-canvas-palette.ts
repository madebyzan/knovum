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
  noteFill: Record<CanvasFillPreset, string>;
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
    measurement: value(styles, "--knovum-space-measurement-color"),
    noteFill: {
      yellow: value(styles, "--knovum-space-note-bg-yellow"),
      green: value(styles, "--knovum-space-note-bg-green"),
      blue: value(styles, "--knovum-space-note-bg-blue"),
      pink: value(styles, "--knovum-space-note-bg-pink"),
      gray: value(styles, "--knovum-space-note-bg-gray"),
    },
    panel: value(styles, "--color-surface-panel"),
    relation: {
      neutral: value(styles, "--color-fg-secondary"),
      blue: value(styles, "--knovum-space-relation-stroke-blue"),
      green: value(styles, "--knovum-space-relation-stroke-green"),
      amber: value(styles, "--knovum-space-relation-stroke-amber"),
      red: value(styles, "--knovum-space-relation-stroke-red"),
      purple: value(styles, "--knovum-space-relation-stroke-purple"),
    },
  };
}

function value(styles: CSSStyleDeclaration, name: string) {
  const resolved = styles.getPropertyValue(name).trim();
  if (!resolved) throw new Error(`Canvas color token ${name} is not defined`);
  return resolved;
}
