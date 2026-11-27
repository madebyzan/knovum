/** Image derivations the node bar and overflow menu invoke directly, and the action each one names. */
export const DIRECT_DERIVATIONS = [
  "remove-image-background",
  "split-image-layers",
  "slice-sticker-sheet",
] as const;

export type DirectImageDerivation = (typeof DIRECT_DERIVATIONS)[number];

export const DERIVATION_LABEL_KEY: Record<DirectImageDerivation, string> = {
  "remove-image-background": "spaces.creation.image.removeBackground",
  "split-image-layers": "spaces.creation.image.splitLayers",
  "slice-sticker-sheet": "spaces.creation.image.sliceStickers",
};
