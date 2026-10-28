export const SECTION_HEADER_HEIGHT = 40;
export const SECTION_DEFAULT_SIZE = { width: 640, height: 400 } as const;
export const SECTION_MINIMUM_SIZE = { width: 240, height: 160 } as const;

interface SectionSize {
  width: number;
  height: number;
}

/** The uniform scale at which `size` just meets the Section minimum on both axes. */
export function sectionMinimumScale(size: SectionSize) {
  return Math.max(SECTION_MINIMUM_SIZE.width / size.width, SECTION_MINIMUM_SIZE.height / size.height);
}

/**
 * `size` held to the Section minimum. With `keepRatio` it grows uniformly, so a locked aspect ratio
 * survives the clamp; otherwise each axis is clamped on its own.
 */
export function atLeastSectionMinimum(size: SectionSize, keepRatio: boolean): SectionSize {
  if (!keepRatio)
    return {
      width: Math.max(SECTION_MINIMUM_SIZE.width, size.width),
      height: Math.max(SECTION_MINIMUM_SIZE.height, size.height),
    };
  const scale = Math.max(1, sectionMinimumScale(size));
  return { width: size.width * scale, height: size.height * scale };
}
