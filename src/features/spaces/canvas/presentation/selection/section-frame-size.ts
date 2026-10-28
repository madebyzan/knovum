import { SECTION_MINIMUM_SIZE } from "../../../model/structure/section-metrics";

export interface SectionSize {
  width: number;
  height: number;
}

/** Common output canvases, in pixels: a Section sized to one of them exports at that size. */
export const SECTION_SIZE_PRESETS = [
  { id: "1:1", width: 1080, height: 1080 },
  { id: "4:5", width: 1080, height: 1350 },
  { id: "9:16", width: 1080, height: 1920 },
  { id: "16:9", width: 1920, height: 1080 },
  { id: "4:3", width: 1600, height: 1200 },
  { id: "A4", width: 2480, height: 3508 },
] as const satisfies readonly (SectionSize & { id: string })[];

type SectionSizePresetId = (typeof SECTION_SIZE_PRESETS)[number]["id"];

export function sectionSizePreset({ width, height }: SectionSize): SectionSizePresetId | undefined {
  return SECTION_SIZE_PRESETS.find((preset) => preset.width === width && preset.height === height)?.id;
}

/**
 * The size an edit of one dimension asks for. With the ratio locked the other dimension follows
 * it, and both are held to the Section minimum without breaking the ratio.
 */
export function editedSectionSize(
  size: SectionSize,
  edit: Partial<SectionSize>,
  lockRatio: boolean,
): SectionSize {
  const requested = { ...size, ...edit };
  if (!lockRatio) return clamp(requested);
  const ratio = size.width / size.height;
  const proportional =
    edit.width === undefined
      ? { width: requested.height * ratio, height: requested.height }
      : { width: requested.width, height: requested.width / ratio };
  const scale = Math.max(
    1,
    SECTION_MINIMUM_SIZE.width / proportional.width,
    SECTION_MINIMUM_SIZE.height / proportional.height,
  );
  return { width: Math.round(proportional.width * scale), height: Math.round(proportional.height * scale) };
}

function clamp({ width, height }: SectionSize): SectionSize {
  return {
    width: Math.max(SECTION_MINIMUM_SIZE.width, Math.round(width)),
    height: Math.max(SECTION_MINIMUM_SIZE.height, Math.round(height)),
  };
}
