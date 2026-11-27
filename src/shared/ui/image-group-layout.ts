const IMAGE_GROUP_LAYOUTS = ["auto", "gallery", "compare", "masonry", "cards", "stack"] as const;
export type ImageGroupLayout = (typeof IMAGE_GROUP_LAYOUTS)[number];

const IMAGE_GROUP_LAYOUT_SET = new Set<string>(IMAGE_GROUP_LAYOUTS);

export function normalizeImageGroupLayout(value: unknown): ImageGroupLayout {
  if (typeof value === "string") {
    const normalized = value.trim().toLowerCase();
    if (IMAGE_GROUP_LAYOUT_SET.has(normalized)) return normalized as ImageGroupLayout;
  }
  return "auto";
}

/** Resolves effective visual column count for auto/gallery/cards/masonry grids. */
export function resolveImageGroupColumns(layout: ImageGroupLayout, count: number): number {
  if (count <= 1 || layout === "stack" || layout === "compare") return 1;
  if (layout === "masonry") return count >= 5 ? 3 : 2;
  return count === 2 || count === 4 ? 2 : 3;
}

export interface LightboxImageItem {
  readonly src: string;
  readonly caption: string;
}

export interface ParsedCardCaption {
  readonly title: string;
  readonly description: string;
}

/** Splits an image's `alt` into optional card title and description (`Title | Description`). */
export function parseImageCardCaption(alt: string): ParsedCardCaption {
  const trimmed = alt.trim();
  if (!trimmed) return { title: "", description: "" };
  const sepIndex = trimmed.indexOf("|");
  if (sepIndex === -1) return { title: trimmed, description: "" };
  return {
    title: trimmed.slice(0, sepIndex).trim(),
    description: trimmed.slice(sepIndex + 1).trim(),
  };
}
