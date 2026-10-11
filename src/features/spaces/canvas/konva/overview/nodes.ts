import Konva from "konva";

import type { CanvasPalette } from "../presentation/use-canvas-palette";
import type { CanvasSceneNode } from "../scene/canvas-scene";

interface CanvasOverviewNodeOptions {
  parent: Konva.Group;
  sceneNode: CanvasSceneNode;
  palette: CanvasPalette;
  scale: number;
  image?: HTMLImageElement;
}

export function drawCanvasOverviewNode(options: CanvasOverviewNodeOptions) {
  const { parent, sceneNode, palette, scale, image } = options;
  const { frame } = sceneNode.layout;
  const { data } = sceneNode.node;
  const group = new Konva.Group({
    x: frame.x,
    y: frame.y,
    rotation: frame.rotation,
    listening: false,
  });
  parent.add(group);

  if (data.kind === "text") {
    drawTextNode(group, sceneNode, palette, scale);
    return;
  }
  drawNodeBackground(group, sceneNode, palette);
  if (image) drawPreviewImage(group, sceneNode, image);
  if (data.kind === "image") return;
  if (frame.width * scale < 38 || frame.height * scale < 24) return;
  if (data.kind === "audio") drawAudioWave(group, frame.width, frame.height, palette);
  drawNodeLabel(group, sceneNode, palette, Boolean(image));
}

function drawTextNode(group: Konva.Group, sceneNode: CanvasSceneNode, palette: CanvasPalette, scale: number) {
  const { frame, appearance } = sceneNode.layout;
  if (frame.width * scale < 28) return;
  const { data } = sceneNode.node;
  if (data.kind !== "text") return;
  group.add(
    new Konva.Text({
      text: (data.plainText.trim() || data.title).slice(0, 300),
      width: frame.width,
      height: frame.height,
      fontSize: (appearance?.fontSize ?? 18) * (appearance?.typographyScale ?? 1),
      fontStyle: appearance?.fontWeight === "bold" ? "bold" : "normal",
      fill: palette.foreground,
      listening: false,
    }),
  );
}

function drawNodeBackground(group: Konva.Group, sceneNode: CanvasSceneNode, palette: CanvasPalette) {
  const { frame, appearance } = sceneNode.layout;
  const note = sceneNode.node.data.kind === "note";
  const fill =
    appearance?.customFill ?? (appearance?.fill ? palette.contentFill[appearance.fill] : undefined);
  group.add(
    new Konva.Rect({
      width: frame.width,
      height: frame.height,
      fill: fill ?? (note ? palette.contentFill.yellow : palette.panel),
      cornerRadius: note ? 3 : 7,
      listening: false,
    }),
  );
}

function drawPreviewImage(group: Konva.Group, sceneNode: CanvasSceneNode, image: HTMLImageElement) {
  const { frame } = sceneNode.layout;
  const imageHeight = sceneNode.node.data.kind === "image" ? frame.height : frame.height * 0.72;
  group.add(
    new Konva.Image({
      image,
      width: frame.width,
      height: imageHeight,
      crop: coverCrop(image, frame.width, imageHeight),
      listening: false,
    }),
  );
}

function drawNodeLabel(
  group: Konva.Group,
  sceneNode: CanvasSceneNode,
  palette: CanvasPalette,
  footer: boolean,
) {
  const { frame, appearance } = sceneNode.layout;
  const { data } = sceneNode.node;
  const note = data.kind === "note";
  const content = note ? noteText(data) : data.title;
  group.add(
    new Konva.Text({
      x: 10,
      y: footer ? frame.height * 0.76 : 10,
      width: frame.width - 20,
      height: footer ? frame.height * 0.22 : frame.height - 20,
      text: content.slice(0, note ? 350 : 140),
      fontFamily: palette.typography.fontFamily,
      fontSize: note ? noteFontSize(appearance) : 14,
      fontStyle: noteFontStyle(note, appearance),
      fill: palette.foreground,
      listening: false,
    }),
  );
}

function noteText(data: Extract<CanvasSceneNode["node"]["data"], { kind: "note" }>) {
  return data.plainText.trim() || data.title;
}

function noteFontSize(appearance: CanvasSceneNode["layout"]["appearance"]) {
  return (appearance?.fontSize ?? 18) * (appearance?.typographyScale ?? 1);
}

function noteFontStyle(note: boolean, appearance: CanvasSceneNode["layout"]["appearance"]) {
  return note && appearance?.fontWeight === "bold" ? "bold" : "normal";
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
