export interface CropPreset {
  id: string;
  ratio?: number;
  labelKey?: string;
  referenceSize?: string;
}

interface CropPresetGroup {
  id: string;
  presets: readonly CropPreset[];
}

export const cropPresetGroups: readonly CropPresetGroup[] = [
  {
    id: "general",
    presets: [
      { id: "original", labelKey: "crop.original" },
      { id: "1:1", ratio: 1 },
      { id: "3:4", ratio: 3 / 4 },
      { id: "2:3", ratio: 2 / 3 },
      { id: "9:16", ratio: 9 / 16 },
      { id: "4:3", ratio: 4 / 3 },
      { id: "3:2", ratio: 3 / 2 },
      { id: "16:9", ratio: 16 / 9 },
    ],
  },
  {
    id: "Instagram",
    presets: [
      { id: "square", labelKey: "crop.square", ratio: 1, referenceSize: "1080×1080" },
      { id: "story", labelKey: "crop.story", ratio: 9 / 16, referenceSize: "1080×1920" },
      { id: "portrait", labelKey: "crop.portrait", ratio: 4 / 5, referenceSize: "1080×1350" },
    ],
  },
  {
    id: "Facebook",
    presets: [
      { id: "square", labelKey: "crop.square", ratio: 1, referenceSize: "1080×1080" },
      { id: "landscape", labelKey: "crop.landscape", ratio: 1200 / 630, referenceSize: "1200×630" },
    ],
  },
  {
    id: "TikTok",
    presets: [{ id: "portrait", labelKey: "crop.portrait", ratio: 9 / 16, referenceSize: "1080×1920" }],
  },
  {
    id: "YouTube",
    presets: [
      { id: "landscape", labelKey: "crop.landscape", ratio: 16 / 9, referenceSize: "1920×1080" },
      { id: "portrait", labelKey: "crop.portrait", ratio: 9 / 16, referenceSize: "1080×1920" },
    ],
  },
  {
    id: "LinkedIn",
    presets: [
      { id: "square", labelKey: "crop.square", ratio: 1, referenceSize: "1200×1200" },
      { id: "landscape", labelKey: "crop.landscape", ratio: 1200 / 627, referenceSize: "1200×627" },
    ],
  },
  {
    id: "Twitter",
    presets: [
      { id: "landscape", labelKey: "crop.landscape", ratio: 16 / 9, referenceSize: "1600×900" },
      { id: "square", labelKey: "crop.square", ratio: 1, referenceSize: "1200×1200" },
    ],
  },
];
