import type { VideoTextStyle } from "./project";

export interface SubtitleStyleTemplate {
  id: "classic" | "cinemaPill" | "highlight" | "softShadow" | "lightPill" | "minimal";
  patch: Partial<VideoTextStyle>;
  previewClass: string;
}

/** Broadcast-grade one-click caption templates modeled after CapCut & cinema subtitling standards. */
export const SUBTITLE_STYLE_TEMPLATES: readonly SubtitleStyleTemplate[] = [
  {
    id: "classic",
    patch: {
      weight: 600,
      italic: false,
      size: 36,
      color: "#FFFFFF",
      decoration: "outline",
      decorationStrength: 2,
      decorationColor: "#000000",
    },
    previewClass: "bg-neutral-800 text-white font-semibold [text-shadow:0_1px_2px_#000,0_0_2px_#000]",
  },
  {
    id: "cinemaPill",
    patch: {
      weight: 600,
      italic: false,
      size: 38,
      color: "#FFFFFF",
      decoration: "box",
      decorationStrength: 7,
      decorationColor: "#000000",
    },
    previewClass: "bg-black/80 text-white font-semibold",
  },
  {
    id: "highlight",
    patch: {
      weight: 700,
      italic: false,
      size: 44,
      color: "#FFD43B",
      decoration: "outline",
      decorationStrength: 4,
      decorationColor: "#000000",
    },
    previewClass: "bg-neutral-800 text-amber-300 font-bold [text-shadow:0_1px_2px_#000,0_0_2px_#000]",
  },
  {
    id: "softShadow",
    patch: {
      weight: 600,
      italic: false,
      size: 42,
      color: "#FFFFFF",
      decoration: "shadow",
      decorationStrength: 4,
      decorationColor: "#000000",
    },
    previewClass: "bg-neutral-700 text-white font-semibold [text-shadow:0_2px_4px_rgba(0,0,0,0.9)]",
  },
  {
    id: "lightPill",
    patch: {
      weight: 600,
      italic: false,
      size: 38,
      color: "#111111",
      decoration: "box",
      decorationStrength: 7,
      decorationColor: "#FFFFFF",
    },
    previewClass: "bg-white/95 text-neutral-900 font-semibold border border-stroke-divider",
  },
  {
    id: "minimal",
    patch: {
      weight: 500,
      italic: false,
      size: 40,
      color: "#FFFFFF",
      decoration: "plain",
      decorationStrength: 0,
      decorationColor: "#000000",
    },
    previewClass: "bg-neutral-800 text-white/95 font-medium",
  },
];
