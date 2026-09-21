import Konva from "konva";

import type { CanvasPalette } from "../presentation/use-canvas-palette";
import type { CanvasSceneNode } from "../scene/canvas-scene";

interface CanvasOverviewNodeOptions {
  parent: Konva.Group;
  item: CanvasSceneNode;
  palette: CanvasPalette;
  scale: number;
  image: HTMLImageElement | undefined;
}

interface CanvasOverviewNodeContext extends Omit<CanvasOverviewNodeOptions, "parent"> {
  group: Konva.Group;
}

export function drawCanvasOverviewNode(options: CanvasOverviewNodeOptions) {
  const { frame } = options.item.layout;
  const group = new Konva.Group({
    x: frame.x,
    y: frame.y,
    rotation: frame.rotation,
    listening: false,
  });
  options.parent.add(group);
  const context = { ...options, group };

  if (options.item.node.data.kind === "section") {
    drawSection(context);
    return;
  }
  if (options.item.node.data.kind === "text") {
    drawText(context);
    return;
  }
  drawCard(context);
}

function drawSection(context: CanvasOverviewNodeContext) {
  const { frame, appearance } = context.item.layout;
  const fill =
    appearance?.customFill ??
    (appearance?.fill ? context.palette.noteFill[appearance.fill] : context.palette.surface);
  context.group.add(
    new Konva.Rect({
      width: frame.width,
      height: frame.height,
      fill,
      opacity: 0.22,
      cornerRadius: 8,
      listening: false,
    }),
  );
}

function drawText(context: CanvasOverviewNodeContext) {
  const { frame, appearance } = context.item.layout;
  const data = context.item.node.data;
  if (data.kind !== "text" || frame.width * context.scale < 28) return;
  context.group.add(
    new Konva.Text({
      text: (data.plainText.trim() || data.title).slice(0, 300),
      width: frame.width,
      height: frame.height,
      fontSize: (appearance?.fontSize ?? 18) * (appearance?.typographyScale ?? 1),
      fontStyle: appearance?.fontWeight === "bold" ? "bold" : "normal",
      fill: context.palette.foreground,
      listening: false,
    }),
  );
}

function drawCard(context: CanvasOverviewNodeContext) {
  const { data } = context.item.node;
  drawCardBackground(context);
  const hasImage = drawPreviewImage(context);
  if (data.kind === "image") return;
  if (!hasReadableLabel(context)) return;
  if (data.kind === "audio") {
    const { frame } = context.item.layout;
    drawAudioWave(context.group, frame.width, frame.height, context.palette);
  }
  drawCardLabel(context, hasImage);
}

function drawCardBackground(context: CanvasOverviewNodeContext) {
  const { frame, appearance } = context.item.layout;
  const note = context.item.node.data.kind === "note";
  const selectedFill =
    appearance?.customFill ?? (appearance?.fill ? context.palette.noteFill[appearance.fill] : undefined);
  context.group.add(
    new Konva.Rect({
      width: frame.width,
      height: frame.height,
      fill: selectedFill ?? (note ? context.palette.noteFill.yellow : context.palette.surface),
      cornerRadius: note ? 3 : 7,
      listening: false,
    }),
  );
}

function drawPreviewImage(context: CanvasOverviewNodeContext) {
  if (!context.image) return false;
  const { frame } = context.item.layout;
  const imageHeight = context.item.node.data.kind === "image" ? frame.height : frame.height * 0.72;
  context.group.add(
    new Konva.Image({
      image: context.image,
      width: frame.width,
      height: imageHeight,
      crop: coverCrop(context.image, frame.width, imageHeight),
      listening: false,
    }),
  );
  return true;
}

function hasReadableLabel(context: CanvasOverviewNodeContext) {
  const { frame } = context.item.layout;
  return frame.width * context.scale >= 38 && frame.height * context.scale >= 24;
}

function drawCardLabel(context: CanvasOverviewNodeContext, hasImage: boolean) {
  const { frame } = context.item.layout;
  const position = cardLabelPosition(frame.height, hasImage);
  const typography = cardLabelTypography(context);
  context.group.add(
    new Konva.Text({
      x: 10,
      y: position.y,
      width: frame.width - 20,
      height: position.height,
      text: cardLabelText(context).slice(0, typography.maxLength),
      fontSize: typography.fontSize,
      fontStyle: typography.fontStyle,
      fill: context.palette.foreground,
      listening: false,
    }),
  );
}

function cardLabelPosition(height: number, hasImage: boolean) {
  return hasImage ? { y: height * 0.76, height: height * 0.22 } : { y: 10, height: height - 20 };
}

function cardLabelText(context: CanvasOverviewNodeContext) {
  const data = context.item.node.data;
  return data.kind === "note" ? data.plainText.trim() || data.title : data.title;
}

function cardLabelTypography(context: CanvasOverviewNodeContext) {
  const { appearance } = context.item.layout;
  if (context.item.node.data.kind !== "note") {
    return { fontSize: 14, fontStyle: "normal" as const, maxLength: 140 };
  }
  return {
    fontSize: (appearance?.fontSize ?? 18) * (appearance?.typographyScale ?? 1),
    fontStyle: appearance?.fontWeight === "bold" ? ("bold" as const) : ("normal" as const),
    maxLength: 350,
  };
}

function coverCrop(image: HTMLImageElement, width: number, height: number) {
  const sourceWidth = image.naturalWidth;
  const sourceHeight = image.naturalHeight;
  const targetAspect = width / height;
  const sourceAspect = sourceWidth / sourceHeight;
  if (sourceAspect > targetAspect) {
    const cropWidth = sourceHeight * targetAspect;
    return { x: (sourceWidth - cropWidth) / 2, y: 0, width: cropWidth, height: sourceHeight };
  }
  const cropHeight = sourceWidth / targetAspect;
  return { x: 0, y: (sourceHeight - cropHeight) / 2, width: sourceWidth, height: cropHeight };
}

function drawAudioWave(group: Konva.Group, width: number, height: number, palette: CanvasPalette) {
  const middle = height * 0.56;
  for (let index = 0; index < 12; index += 1) {
    const barHeight = 8 + ((index * 7) % 17);
    group.add(
      new Konva.Rect({
        x: width * 0.17 + index * (width * 0.055),
        y: middle - barHeight / 2,
        width: Math.max(2, width * 0.018),
        height: barHeight,
        fill: palette.foregroundSecondary,
        cornerRadius: 2,
        listening: false,
      }),
    );
  }
}
