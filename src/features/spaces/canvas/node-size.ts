import { pick } from "es-toolkit";

import { rotatedImageSize } from "../image-editing/image-edit";
import type { SpaceNodePayload, SpaceNodeKind } from "../model/nodes/node-payload";

export type SpaceNodeResizeMode = "fixed" | "free" | "proportional" | "width-and-proportional";

export interface SpaceNodeSizePolicy {
  width: number;
  height: number;
  minWidth: number;
  minHeight: number;
  maxWidth: number;
  maxHeight: number;
  resizeMode: SpaceNodeResizeMode;
  contentAspect?: {
    ratio: number;
    fixedVerticalChrome: number;
  };
}

export const PREVIEW_CAPTION_HEIGHT = 48;
export const CANVAS_NODE_HEADER_HEIGHT = 40;
export const DEFAULT_CANVAS_TEXT_FONT_SIZE = 18;

const SIZE_POLICY_BY_NODE_KIND: Record<SpaceNodeKind, SpaceNodeSizePolicy> = {
  note: policy([192, 192], [144, 112]),
  text: policy([220, 40], [96, 40], "width-and-proportional"),
  markdown: policy([260, 184], [224, 144]),
  website: policy([288, 208], [248, 168], "proportional"),
  image: policy([256, 156], [144, 72], "proportional"),
  video: policy([304, 170], [232, 132], "proportional"),
  audio: policy([288, 80], [288, 80], "fixed"),
  file: policy([256, 192], [224, 152], "proportional"),
  code: policy([304, 224], [264, 168]),
  credential: policy([220, 56], [220, 56], "fixed"),
  object: policy([256, 176], [224, 144]),
  portal: policy([256, 88], [256, 88], "fixed"),
};

export function spaceNodeDefaultDimensions(kind: SpaceNodeKind) {
  return pick(SIZE_POLICY_BY_NODE_KIND[kind], ["width", "height"]);
}

export function spaceNodeSizePolicy(data: SpaceNodePayload): SpaceNodeSizePolicy {
  const base = SIZE_POLICY_BY_NODE_KIND[data.kind];
  const ratio = spaceVisualDisplayRatio(data);
  if (ratio && data.kind === "image") return { ...base, ...fitImageNode(ratio) };
  if (ratio && data.kind === "video") return { ...base, ...fitVideoNode(ratio) };
  const page = spacePageDisplayRatio(data);
  return page ? { ...base, ...fitDocumentNode(page) } : base;
}

export function spaceNodeDimensions(data: SpaceNodePayload) {
  const size = spaceNodeSizePolicy(data);
  return { width: size.width, height: size.height };
}

export function spaceNodeSupportsContentSizing(
  data: SpaceNodePayload,
): data is Extract<SpaceNodePayload, { kind: "image" | "video" | "file" }> {
  return data.kind === "image" || data.kind === "video" || data.kind === "file";
}

export function spaceNodeUsesContentFitHeight(data: SpaceNodePayload) {
  return data.kind === "text";
}

export function spaceNodeMinimumContentHeight(data: SpaceNodePayload, typographyScale = 1) {
  const minimum = spaceNodeSizePolicy(data).minHeight;
  return data.kind === "text" ? minimum * typographyScale : minimum;
}

export function spaceNodeResizePolicy(data: SpaceNodePayload) {
  return pick(spaceNodeSizePolicy(data), [
    "minWidth",
    "minHeight",
    "maxWidth",
    "maxHeight",
    "resizeMode",
    "contentAspect",
  ]);
}

function spaceVisualDisplayRatio(data: SpaceNodePayload) {
  if (data.kind !== "image" && data.kind !== "video") return undefined;
  if (!data.intrinsicWidth || !data.intrinsicHeight) return undefined;
  if (data.kind === "video") return data.intrinsicWidth / data.intrinsicHeight;
  return editedImageDisplayRatio(data, data.intrinsicWidth, data.intrinsicHeight);
}

function editedImageDisplayRatio(
  data: Extract<SpaceNodePayload, { kind: "image" }>,
  intrinsicWidth: number,
  intrinsicHeight: number,
) {
  const crop = data.imageEdit?.crop;
  const width = intrinsicWidth * (crop?.width ?? 1);
  const height = intrinsicHeight * (crop?.height ?? 1);
  const rotated = rotatedImageSize(width, height, data.imageEdit?.rotation ?? 0);
  return rotated.width / rotated.height;
}

function spacePageDisplayRatio(data: SpaceNodePayload) {
  if (data.kind !== "file") return undefined;
  if (!data.intrinsicWidth || !data.intrinsicHeight) return undefined;
  return data.intrinsicWidth / data.intrinsicHeight;
}

function policy(
  [width, height]: readonly [number, number],
  [minWidth, minHeight]: readonly [number, number],
  resizeMode: SpaceNodeSizePolicy["resizeMode"] = "free",
): SpaceNodeSizePolicy {
  const maxWidth = resizeMode === "fixed" ? width : Number.POSITIVE_INFINITY;
  const maxHeight = resizeMode === "fixed" ? height : Number.POSITIVE_INFINITY;
  return { width, height, minWidth, minHeight, maxWidth, maxHeight, resizeMode };
}

function fitImageNode(ratio: number) {
  const preview = fitIntrinsicRatio(ratio, 280, 128, 480);
  return previewSizePolicy(preview, 0);
}

function fitVideoNode(ratio: number) {
  return previewSizePolicy(fitIntrinsicRatio(ratio, 304, 156, 560), 0);
}

function fitDocumentNode(ratio: number) {
  return previewSizePolicy(fitIntrinsicRatio(ratio, 272, 150, 480));
}

function fitIntrinsicRatio(
  ratio: number,
  preferredLongEdge: number,
  minimumShortEdge: number,
  maxLongEdge: number,
) {
  let width = ratio >= 1 ? preferredLongEdge : preferredLongEdge * ratio;
  let height = ratio >= 1 ? preferredLongEdge / ratio : preferredLongEdge;
  const shortEdge = Math.min(width, height);
  if (shortEdge < minimumShortEdge) {
    const scale = minimumShortEdge / shortEdge;
    width *= scale;
    height *= scale;
  }
  const longEdge = Math.max(width, height);
  if (longEdge > maxLongEdge) {
    const scale = maxLongEdge / longEdge;
    width *= scale;
    height *= scale;
  }
  return { width, height };
}

function previewSizePolicy(
  preview: { width: number; height: number },
  captionHeight = PREVIEW_CAPTION_HEIGHT,
) {
  const width = Math.round(preview.width);
  const height = Math.round(preview.height + captionHeight);
  return {
    width,
    height,
    minWidth: Math.min(width, 200),
    minHeight: Math.min(height, 164),
    maxWidth: Number.POSITIVE_INFINITY,
    maxHeight: Number.POSITIVE_INFINITY,
    contentAspect: {
      ratio: preview.width / preview.height,
      fixedVerticalChrome: captionHeight,
    },
  };
}
