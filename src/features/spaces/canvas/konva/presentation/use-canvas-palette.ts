import { useMutationObserver } from "@vueuse/core";
import { onMounted, shallowRef } from "vue";

import type { CanvasNodeAppearance, CanvasRelationColor } from "../../contract/canvas-document";

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

const INITIAL: CanvasPalette = {
  accent: "#3978f6",
  border: "rgba(20, 22, 27, 0.08)",
  borderStrong: "rgba(20, 22, 27, 0.16)",
  canvas: "#fafafb",
  foreground: "#242529",
  foregroundSecondary: "#6f7178",
  grid: "rgba(36, 37, 41, 0.255)",
  measurement: "oklch(0.72 0.19 72)",
  noteFill: {
    yellow: "#fde68a",
    green: "#a7f3d0",
    blue: "#bae6fd",
    pink: "#fecdd3",
    gray: "#e4e4e7",
  },
  surface: "#f7f7f8",
  relation: {
    neutral: "#6f7178",
    blue: "oklch(0.62 0.2 255)",
    green: "oklch(0.61 0.16 148)",
    amber: "oklch(0.69 0.16 72)",
    red: "oklch(0.61 0.22 28)",
    purple: "oklch(0.62 0.18 305)",
  },
};

export function useCanvasPalette() {
  const palette = shallowRef(INITIAL);

  function read() {
    const styles = getComputedStyle(document.documentElement);
    palette.value = {
      accent: value(styles, "--color-accent"),
      border: value(styles, "--color-border"),
      borderStrong: value(styles, "--color-border-strong"),
      canvas: value(styles, "--color-canvas"),
      foreground: value(styles, "--color-foreground"),
      foregroundSecondary: value(styles, "--color-foreground-secondary"),
      grid: value(styles, "--knovum-canvas-grid-color"),
      measurement: value(styles, "--knovum-canvas-measurement-color"),
      noteFill: {
        yellow: value(styles, "--knovum-canvas-note-yellow"),
        green: value(styles, "--knovum-canvas-note-green"),
        blue: value(styles, "--knovum-canvas-note-blue"),
        pink: value(styles, "--knovum-canvas-note-pink"),
        gray: value(styles, "--knovum-canvas-note-gray"),
      },
      surface: value(styles, "--color-surface"),
      relation: {
        neutral: value(styles, "--color-foreground-secondary"),
        blue: value(styles, "--knovum-canvas-relation-blue"),
        green: value(styles, "--knovum-canvas-relation-green"),
        amber: value(styles, "--knovum-canvas-relation-amber"),
        red: value(styles, "--knovum-canvas-relation-red"),
        purple: value(styles, "--knovum-canvas-relation-purple"),
      },
    };
  }

  onMounted(read);
  useMutationObserver(document.documentElement, read, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
  return palette;
}

function value(styles: CSSStyleDeclaration, name: string) {
  const resolved = styles.getPropertyValue(name).trim();
  if (!resolved) throw new Error(`Canvas color token ${name} is not defined`);
  return resolved;
}
