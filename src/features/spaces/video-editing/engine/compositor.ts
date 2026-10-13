import { layerGeometry } from "../geometry";
import type {
  VideoAsset,
  VideoOutput,
  VideoTextOverlay,
  VideoTransform,
  VideoTransitionKind,
} from "../project";
import { drawText, textBounds } from "./text-draw";
import { drawTransition } from "./transition-draw";

export interface CompositionLayer {
  media: CanvasImageSource;
  asset: VideoAsset;
  transform: VideoTransform;
  /** Visual fade gain; the frame dips to black exactly like the exported fade. */
  fade: number;
}

export interface CompositionFrame {
  output: VideoOutput;
  layers: CompositionLayer[];
  transition?: { kind: VideoTransitionKind; progress: number } | undefined;
  texts: VideoTextOverlay[];
}

/**
 * Paints one output frame. Layers are composed over the project background exactly as the export
 * filter graph does, so the stage is a preview of the render rather than a different renderer.
 */
export function createCompositor(canvas: HTMLCanvasElement) {
  const context = canvas.getContext("2d", { alpha: false });
  const buffers = [createBuffer(), createBuffer()] as const;

  function resize(output: VideoOutput) {
    for (const surface of [canvas, buffers[0].canvas, buffers[1].canvas]) {
      if (surface.width === output.width && surface.height === output.height) continue;
      surface.width = output.width;
      surface.height = output.height;
    }
  }

  function draw(frame: CompositionFrame) {
    if (!context) return;
    resize(frame.output);
    context.imageSmoothingQuality = "high";
    const [outgoing, incoming] = frame.layers;
    if (frame.transition && outgoing && incoming) {
      paintLayer(buffers[0].context, outgoing, frame.output);
      paintLayer(buffers[1].context, incoming, frame.output);
      drawTransition(context, {
        kind: frame.transition.kind,
        frames: { from: buffers[0].canvas, to: buffers[1].canvas },
        progress: frame.transition.progress,
        size: frame.output,
      });
    } else if (incoming ?? outgoing) {
      paintLayer(context, (incoming ?? outgoing)!, frame.output);
    } else {
      paintBackground(context, frame.output);
    }
    for (const item of frame.texts) drawText(context, item, frame.output);
  }

  return { draw, resize, context, measureText };

  function measureText(item: VideoTextOverlay, output: VideoOutput) {
    return context ? textBounds(context, item, output) : undefined;
  }
}

function paintBackground(context: CanvasRenderingContext2D, output: VideoOutput) {
  context.globalAlpha = 1;
  context.fillStyle = output.background;
  context.fillRect(0, 0, output.width, output.height);
}

function paintLayer(context: CanvasRenderingContext2D, layer: CompositionLayer, output: VideoOutput) {
  paintBackground(context, output);
  const geometry = layerGeometry(layer.asset, layer.transform, output);
  context.save();
  context.translate(geometry.centerX, geometry.centerY);
  context.rotate((layer.transform.rotation * Math.PI) / 180);
  context.scale(layer.transform.flipHorizontal ? -1 : 1, layer.transform.flipVertical ? -1 : 1);
  context.globalAlpha = layer.transform.opacity;
  context.drawImage(layer.media, -geometry.width / 2, -geometry.height / 2, geometry.width, geometry.height);
  context.restore();
  context.globalAlpha = 1;
  if (layer.fade < 1) {
    context.fillStyle = "#000000";
    context.globalAlpha = 1 - layer.fade;
    context.fillRect(0, 0, output.width, output.height);
    context.globalAlpha = 1;
  }
}

export type Compositor = ReturnType<typeof createCompositor>;

function createBuffer() {
  const buffer = document.createElement("canvas");
  return { canvas: buffer, context: buffer.getContext("2d", { alpha: false })! };
}
