import Konva from "konva";

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasPalette } from "../rendering/use-canvas-palette";
import type { CanvasScene } from "../scene/canvas-scene";
import { restingRelations } from "../scene/scene-relations";
import { loadCanvasOverviewImages } from "./assets";
import { canvasOverviewViewport, canvasViewportThumbnail } from "./geometry";
import { drawCanvasOverviewNode } from "./nodes";

const WIDTH = 480;
const HEIGHT = 270;

export async function captureCanvasOverview(scene: CanvasScene, palette: CanvasPalette): Promise<Blob> {
  const viewport = canvasOverviewViewport(scene, WIDTH, HEIGHT);
  return renderCanvasThumbnail(scene, palette, viewport);
}

interface CanvasViewportCaptureInput {
  scene: CanvasScene;
  palette: CanvasPalette;
  viewport: CanvasViewport;
  size: { width: number; height: number };
}

export function captureCanvasViewport({
  scene,
  palette,
  viewport,
  size,
}: CanvasViewportCaptureInput): Promise<Blob> {
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
  const images = await loadCanvasOverviewImages(scene, viewport.scale, {
    left: -viewport.x / viewport.scale,
    top: -viewport.y / viewport.scale,
    right: (WIDTH - viewport.x) / viewport.scale,
    bottom: (HEIGHT - viewport.y) / viewport.scale,
  });
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
            (appearance?.fill ? palette.contentFillByPreset[appearance.fill] : palette.panel),
          opacity: 0.26,
          cornerRadius: 12,
          listening: false,
        }),
      );
    }
    for (const relation of restingRelations(scene.relations)) {
      world.add(
        new Konva.Line({
          points: [...relation.points],
          bezier: true,
          stroke: palette.relationByColor[relation.color],
          strokeWidth: 1.5 / viewport.scale,
          opacity: 0.75,
          listening: false,
        }),
      );
    }
    for (const sceneNode of scene.nodes) {
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
