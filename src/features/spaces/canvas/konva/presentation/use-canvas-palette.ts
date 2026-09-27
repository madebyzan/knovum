import { useEventListener, useMutationObserver } from "@vueuse/core";
import { onMounted, shallowRef } from "vue";

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
  surface: string;
  relation: Record<CanvasRelationColor, string>;
}

export function useCanvasPalette() {
  const root = document.documentElement;
  // CSS is already loaded before the canvas mounts. Missing tokens are an error, not a second theme.
  const palette = shallowRef(readPalette(getComputedStyle(root)));
  const refresh = () => {
    palette.value = readPalette(getComputedStyle(root));
  };

  onMounted(refresh);
  useMutationObserver(root, refresh, {
    attributes: true,
    attributeFilter: ["data-theme", "data-window-active"],
  });
  useEventListener(window.matchMedia("(prefers-contrast: more)"), "change", refresh);
  useEventListener(window.matchMedia("(prefers-reduced-transparency: reduce)"), "change", refresh);
  return palette;
}

function readPalette(styles: CSSStyleDeclaration): CanvasPalette {
  return {
    accent: value(styles, "--color-accent"),
    border: value(styles, "--color-border"),
    borderStrong: value(styles, "--color-border-strong"),
    canvas: value(styles, "--color-canvas"),
    foreground: value(styles, "--color-foreground"),
    foregroundSecondary: value(styles, "--color-foreground-secondary"),
    grid: value(styles, "--knovum-canvas-grid-color"),
    measurement: value(styles, "--knovum-canvas-measurement-color"),
    noteFill: {
      yellow: value(styles, "--color-canvas-note-yellow"),
      green: value(styles, "--color-canvas-note-green"),
      blue: value(styles, "--color-canvas-note-blue"),
      pink: value(styles, "--color-canvas-note-pink"),
      gray: value(styles, "--color-canvas-note-gray"),
    },
    surface: value(styles, "--color-surface"),
    relation: {
      neutral: value(styles, "--color-foreground-secondary"),
      blue: value(styles, "--color-canvas-relation-blue"),
      green: value(styles, "--color-canvas-relation-green"),
      amber: value(styles, "--color-canvas-relation-amber"),
      red: value(styles, "--color-canvas-relation-red"),
      purple: value(styles, "--color-canvas-relation-purple"),
    },
  };
}

function value(styles: CSSStyleDeclaration, name: string): string {
  const resolved = styles.getPropertyValue(name).trim();
  if (!resolved) throw new Error(`Canvas color token ${name} is not defined`);
  return resolved;
}
