import Konva from "konva";

import { loadCanvasOverviewImages } from "./canvas-overview-assets";
import { canvasOverviewViewport, canvasViewportThumbnail } from "./canvas-overview-geometry";
import { drawCanvasOverviewNode } from "./canvas-overview-nodes";
import type { CanvasViewport } from "./contract/canvas-editor-port";
import type { CanvasPalette } from "./konva/presentation/use-canvas-palette";
import type { CanvasScene } from "./konva/scene/scene-projection";

const WIDTH = 480;
const HEIGHT = 270;

export async function captureCanvasOverview(scene: CanvasScene, palette: CanvasPalette): Promise<Blob> {
  const viewport = canvasOverviewViewport(scene, WIDTH, HEIGHT);
  return renderCanvasThumbnail(scene, palette, viewport);
}

export function captureCanvasViewport(
  scene: CanvasScene,
  palette: CanvasPalette,
  viewport: CanvasViewport,
  size: { width: number; height: number },
): Promise<Blob> {
  return renderCanvasThumbnail(
    scene,
    palette,
    canvasViewportThumbnail(viewport, size, { width: WIDTH, height: HEIGHT }),
  );
}

async function renderCanvasThumbnail(
  scene: CanvasScene,
  palette: CanvasPalette,
  viewport: { x: number; y: number; scale: number },
): Promise<Blob> {
  const images = await loadCanvasOverviewImages(scene, viewport.scale);
  const host = document.createElement("div");
  host.style.position = "fixed";
  host.style.left = "-10000px";
  host.style.top = "0";
  document.body.append(host);
  const stage = new Konva.Stage({ container: host, width: WIDTH, height: HEIGHT, listening: false });
  try {
    const layer = new Konva.Layer({ listening: false });
    stage.add(layer);
    layer.add(
      new Konva.Rect({
        width: WIDTH,
        height: HEIGHT,
        fill: palette.canvas,
        listening: false,
      }),
    );
    const world = new Konva.Group({
      x: viewport.x,
      y: viewport.y,
      scaleX: viewport.scale,
      scaleY: viewport.scale,
      listening: false,
    });
    layer.add(world);
    for (const section of scene.sectionBackdrops) {
      const appearance = section.appearance;
      world.add(
        new Konva.Rect({
          ...section.bounds,
          fill:
            appearance?.customFill ??
            (appearance?.fill ? palette.noteFill[appearance.fill] : palette.surface),
          opacity: 0.26,
          cornerRadius: 8,
          listening: false,
        }),
      );
    }
    for (const relation of scene.relations) {
      world.add(
        new Konva.Line({
          points: [...relation.points],
          bezier: true,
          stroke: palette.relation[relation.color],
          strokeWidth: 1.5 / viewport.scale,
          opacity: 0.75,
          listening: false,
        }),
      );
    }
    for (const sceneNode of scene.contentNodes) {
      const image = images.get(sceneNode.node.id);
      drawCanvasOverviewNode({
        parent: world,
        sceneNode,
        palette,
        scale: viewport.scale,
        ...(image ? { image } : {}),
      });
    }
    layer.draw();
    return stage.toBlob({ mimeType: "image/jpeg", quality: 0.86, pixelRatio: 1 });
  } finally {
    stage.destroy();
    host.remove();
  }
}
