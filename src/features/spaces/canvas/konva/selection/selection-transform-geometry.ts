import type Konva from "konva";

import { isProportionalResizeAnchor } from "./canvas-resize-anchors";
import type { CanvasSelectionTransformPolicy } from "./selection-transform-policy";

type TransformBox = Parameters<NonNullable<Konva.TransformerConfig["boundBoxFunc"]>>[1];

interface SelectionTransformInput {
  currentBox: TransformBox;
  box: TransformBox;
  activeAnchor: string | null;
  policy: CanvasSelectionTransformPolicy;
  stageScale: Konva.Vector2d;
}

interface ResizeGeometry {
  width: number;
  height: number;
  currentWidth: number;
  currentHeight: number;
}

interface ConstrainedSizeInput {
  geometry: ResizeGeometry;
  activeAnchor: string | null;
  policy: CanvasSelectionTransformPolicy;
}

export function constrainSelectionTransform(input: SelectionTransformInput) {
  const { currentBox, box, activeAnchor, policy, stageScale } = input;
  const geometry = resizeGeometry(currentBox, box, stageScale);
  const size = constrainedSize({ geometry, activeAnchor, policy });
  const nextWidth = Math.sign(box.width) * size.width * stageScale.x;
  const nextHeight = Math.sign(box.height) * size.height * stageScale.y;
  return {
    ...box,
    x: activeAnchor?.includes("left") ? box.x + box.width - nextWidth : box.x,
    y: activeAnchor?.includes("top") ? box.y + box.height - nextHeight : box.y,
    width: nextWidth,
    height: nextHeight,
  };
}

function resizeGeometry(
  currentBox: TransformBox,
  box: TransformBox,
  stageScale: Konva.Vector2d,
): ResizeGeometry {
  return {
    width: Math.abs(box.width / stageScale.x),
    height: Math.abs(box.height / stageScale.y),
    currentWidth: Math.abs(currentBox.width / stageScale.x),
    currentHeight: Math.abs(currentBox.height / stageScale.y),
  };
}

function constrainedSize({ geometry, activeAnchor, policy }: ConstrainedSizeInput) {
  if (policy.contentAspect) return contentAspectSize(geometry, policy);
  if (isProportionalResizeAnchor(policy.resizeMode, activeAnchor)) {
    const baseline = policy.proportionalScale;
    const ratio = baseline
      ? baseline.baseWidth / baseline.baseHeight
      : geometry.currentWidth / geometry.currentHeight;
    return proportionalSize(proportionalGestureWidth(geometry.width, geometry.height, policy), ratio, policy);
  }
  return {
    width: clamp(geometry.width, policy.minWidth, policy.maxWidth),
    height:
      policy.resizeMode === "width-and-proportional"
        ? geometry.currentHeight
        : clamp(geometry.height, policy.minHeight, policy.maxHeight),
  };
}

function contentAspectSize(geometry: ResizeGeometry, policy: CanvasSelectionTransformPolicy) {
  const aspect = policy.contentAspect!;
  const minimumWidth = Math.max(
    policy.minWidth,
    Math.max(0, policy.minHeight - aspect.fixedVerticalChrome) * aspect.ratio,
  );
  const maximumWidth = Math.min(
    policy.maxWidth,
    Math.max(0, policy.maxHeight - aspect.fixedVerticalChrome) * aspect.ratio,
  );
  const contentHeight = Math.max(0, geometry.height - aspect.fixedVerticalChrome);
  const currentContentHeight = Math.max(0, geometry.currentHeight - aspect.fixedVerticalChrome);
  const squaredLength = geometry.currentWidth ** 2 + currentContentHeight ** 2;
  const gestureScale =
    squaredLength === 0
      ? 1
      : (geometry.width * geometry.currentWidth + contentHeight * currentContentHeight) / squaredLength;
  const constrainedWidth = clamp(geometry.currentWidth * gestureScale, minimumWidth, maximumWidth);
  return {
    width: constrainedWidth,
    height: constrainedWidth / aspect.ratio + aspect.fixedVerticalChrome,
  };
}

function proportionalGestureWidth(width: number, height: number, policy: CanvasSelectionTransformPolicy) {
  const baseline = policy.proportionalScale;
  if (!baseline) return width;
  const squaredLength = baseline.baseWidth ** 2 + baseline.baseHeight ** 2;
  const scale = (width * baseline.baseWidth + height * baseline.baseHeight) / squaredLength;
  return baseline.baseWidth * scale;
}

function proportionalSize(width: number, ratio: number, policy: CanvasSelectionTransformPolicy) {
  const scale = policy.proportionalScale;
  if (scale) {
    const constrainedWidth = clamp(width, scale.baseWidth * scale.min, scale.baseWidth * scale.max);
    return { width: constrainedWidth, height: constrainedWidth / ratio };
  }
  const minimumWidth = Math.max(policy.minWidth, policy.minHeight * ratio);
  const maximumWidth = Math.min(policy.maxWidth, policy.maxHeight * ratio);
  const constrainedWidth = clamp(width, minimumWidth, maximumWidth);
  return { width: constrainedWidth, height: constrainedWidth / ratio };
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(maximum, Math.max(minimum, value));
}
