import Konva from "konva";

import type { CanvasPalette } from "./konva/presentation/use-canvas-palette";
import type { CanvasSceneNode } from "./konva/scene/scene-projection";

export function drawCanvasOverviewNode(
  parent: Konva.Group,
  item: CanvasSceneNode,
  palette: CanvasPalette,
  scale: number,
  image?: HTMLImageElement,
) {
  const { frame, appearance } = item.layout;
  const { data } = item.node;
  const group = new Konva.Group({
    x: frame.x,
    y: frame.y,
    rotation: frame.rotation,
    listening: false,
  });
  parent.add(group);

  if (data.kind === "section") {
    group.add(
      new Konva.Rect({
        width: frame.width,
        height: frame.height,
        fill:
          appearance?.customFill ?? (appearance?.fill ? palette.noteFill[appearance.fill] : palette.surface),
        opacity: 0.22,
        cornerRadius: 8,
        listening: false,
      }),
    );
    return;
  }
  if (data.kind === "text") {
    const content = data.plainText.trim() || data.title;
    if (frame.width * scale >= 28) {
      group.add(
        new Konva.Text({
          text: content.slice(0, 300),
          width: frame.width,
          height: frame.height,
          fontSize: (appearance?.fontSize ?? 18) * (appearance?.typographyScale ?? 1),
          fontStyle: appearance?.fontWeight === "bold" ? "bold" : "normal",
          fill: palette.foreground,
          listening: false,
        }),
      );
    }
    return;
  }

  const note = data.kind === "note";
  const fill = appearance?.customFill ?? (appearance?.fill ? palette.noteFill[appearance.fill] : undefined);
  group.add(
    new Konva.Rect({
      width: frame.width,
      height: frame.height,
      fill: fill ?? (note ? palette.noteFill.yellow : palette.surface),
      cornerRadius: note ? 3 : 7,
      listening: false,
    }),
  );

  if (image) {
    const imageHeight = data.kind === "image" ? frame.height : frame.height * 0.72;
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
  if (data.kind === "image") return;
  if (frame.width * scale < 38 || frame.height * scale < 24) return;

  if (data.kind === "audio") drawAudioWave(group, frame.width, frame.height, palette);
  const content = note ? data.plainText.trim() || data.title : data.title;
  const footer = Boolean(image);
  group.add(
    new Konva.Text({
      x: 10,
      y: footer ? frame.height * 0.76 : 10,
      width: frame.width - 20,
      height: footer ? frame.height * 0.22 : frame.height - 20,
      text: content.slice(0, note ? 350 : 140),
      fontSize: note ? (appearance?.fontSize ?? 18) * (appearance?.typographyScale ?? 1) : 14,
      fontStyle: note && appearance?.fontWeight === "bold" ? "bold" : "normal",
      fill: palette.foreground,
      listening: false,
    }),
  );
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
