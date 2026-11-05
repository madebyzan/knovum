import { pick } from "es-toolkit";

import { contentFootprints } from "../contracts/generated/contract-values";
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

export const PREVIEW_CAPTION_HEIGHT = contentFootprints.file.caption;
export const CANVAS_NODE_HEADER_HEIGHT = 40;
export const DEFAULT_CANVAS_TEXT_FONT_SIZE = 10;

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
  const extent = displayExtent(data);
  return extent ? { ...base, ...previewSizePolicy(extent) } : base;
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

type PicturedKind = keyof typeof contentFootprints;

interface DisplayExtent {
  kind: PicturedKind;
  width: number;
  height: number;
}

type PicturedPayload = Extract<SpaceNodePayload, { kind: PicturedKind }>;

/** The size pictured content shows itself at: an image's cropped, rotated pixels, a video's frame,
 * a rendered document's page. */
function displayExtent(data: SpaceNodePayload): DisplayExtent | undefined {
  if (!isPictured(data) || !data.intrinsicWidth || !data.intrinsicHeight) return undefined;
  const intrinsic = { width: data.intrinsicWidth, height: data.intrinsicHeight };
  const shown = data.kind === "image" ? editedImageSize(data, intrinsic) : intrinsic;
  return { kind: data.kind, ...shown };
}

function isPictured(data: SpaceNodePayload): data is PicturedPayload {
  return Object.hasOwn(contentFootprints, data.kind);
}

function editedImageSize(
  data: Extract<SpaceNodePayload, { kind: "image" }>,
  { width, height }: { width: number; height: number },
) {
  const crop = data.imageEdit?.crop;
  return rotatedImageSize(
    width * (crop?.width ?? 1),
    height * (crop?.height ?? 1),
    data.imageEdit?.rotation ?? 0,
  );
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

/** The footprint Rust gives content nobody has sized (`nodes::content_size::default_size`). */
function defaultFootprint({ kind, width: pixelWidth, height: pixelHeight }: DisplayExtent) {
  const footprint = contentFootprints[kind];
  const ratio = pixelWidth / pixelHeight;
  const long = footprint.preferredLongEdge;
  const width = ratio >= 1 ? long : long * ratio;
  const height = ratio >= 1 ? long / ratio : long;
  const grown = Math.max(footprint.minimumShortEdge / Math.min(width, height), 1);
  const scale = Math.min(
    grown,
    footprint.maximumLongEdge / Math.max(width, height),
    footprint.pixelBound ? pixelWidth / width : Number.POSITIVE_INFINITY,
  );
  return { width: width * scale, height: height * scale, caption: footprint.caption };
}

function previewSizePolicy(extent: DisplayExtent) {
  const { caption: captionHeight, ...preview } = defaultFootprint(extent);
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
