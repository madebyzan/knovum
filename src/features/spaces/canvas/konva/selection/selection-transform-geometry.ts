import { clamp } from "es-toolkit";
import type Konva from "konva";

import { isProportionalResizeAnchor } from "./canvas-resize-anchors";
import type { CanvasSelectionTransformPolicy } from "./selection-transform-policy";

type TransformBox = Parameters<NonNullable<Konva.TransformerConfig["boundBoxFunc"]>>[1];

export function constrainSelectionTransform(
  currentBox: TransformBox,
  box: TransformBox,
  options: {
    activeAnchor: string | null;
    policy: CanvasSelectionTransformPolicy;
    stageScale: Konva.Vector2d;
  },
) {
  const { activeAnchor, policy, stageScale } = options;
  const width = Math.abs(box.width / stageScale.x);
  const height = Math.abs(box.height / stageScale.y);
  const currentWidth = Math.abs(currentBox.width / stageScale.x);
  const currentHeight = Math.abs(currentBox.height / stageScale.y);
  const size = constrainedSize({ width, height, currentWidth, currentHeight, activeAnchor, policy });
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

function constrainedSize(options: {
  width: number;
  height: number;
  currentWidth: number;
  currentHeight: number;
  activeAnchor: string | null;
  policy: CanvasSelectionTransformPolicy;
}) {
  const { width, height, currentWidth, currentHeight, activeAnchor, policy } = options;
  if (policy.contentAspect) {
    return contentAspectSize({ width, height, currentWidth, currentHeight, policy });
  }
  if (isProportionalResizeAnchor(policy.resizeMode, activeAnchor)) {
    const baseline = policy.proportionalScale;
    const ratio = baseline ? baseline.baseWidth / baseline.baseHeight : currentWidth / currentHeight;
    return proportionalSize(proportionalGestureWidth(width, height, policy), ratio, policy);
  }
  return {
    width: clamp(width, policy.minWidth, policy.maxWidth),
    height:
      policy.resizeMode === "width-and-proportional"
        ? currentHeight
        : clamp(height, policy.minHeight, policy.maxHeight),
  };
}

function contentAspectSize(options: {
  width: number;
  height: number;
  currentWidth: number;
  currentHeight: number;
  policy: CanvasSelectionTransformPolicy;
}) {
  const { width, height, currentWidth, currentHeight, policy } = options;
  const aspect = policy.contentAspect!;
  const minimumWidth = Math.max(
    policy.minWidth,
    Math.max(0, policy.minHeight - aspect.fixedVerticalChrome) * aspect.ratio,
  );
  const maximumWidth = Math.min(
    policy.maxWidth,
    Math.max(0, policy.maxHeight - aspect.fixedVerticalChrome) * aspect.ratio,
  );
  const contentHeight = Math.max(0, height - aspect.fixedVerticalChrome);
  const currentContentHeight = Math.max(0, currentHeight - aspect.fixedVerticalChrome);
  const squaredLength = currentWidth ** 2 + currentContentHeight ** 2;
  const gestureScale =
    squaredLength === 0 ? 1 : (width * currentWidth + contentHeight * currentContentHeight) / squaredLength;
  const constrainedWidth = clamp(currentWidth * gestureScale, minimumWidth, maximumWidth);
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
