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
 * filter graph does, so the stage previews the render rather than approximating it. The surface is
 * sized to the pixels it actually occupies on screen while every coordinate stays in output space,
 * so a 19 % stage costs a 19 % fill instead of a full 4K composite per frame.
 */
export function createCompositor(canvas: HTMLCanvasElement) {
  const context = canvas.getContext("2d", { alpha: false });
  const buffers = [createBuffer(), createBuffer()] as const;
  const measuring = createBuffer().context;

  function surfaceScale(output: VideoOutput) {
    const ratio = globalThis.devicePixelRatio || 1;
    const width = canvas.clientWidth || output.width;
    return Math.max(0.05, Math.min(1, (width * ratio) / output.width));
  }

  function resize(output: VideoOutput, scale: number) {
    const width = Math.max(2, Math.round(output.width * scale));
    const height = Math.max(2, Math.round(output.height * scale));
    for (const surface of [canvas, buffers[0].canvas, buffers[1].canvas]) {
      if (surface.width === width && surface.height === height) continue;
      surface.width = width;
      surface.height = height;
    }
    return { width, height };
  }

  function draw(frame: CompositionFrame) {
    if (!context) return;
    const scale = surfaceScale(frame.output);
    const size = resize(frame.output, scale);
    for (const surface of [context, buffers[0].context, buffers[1].context]) {
      surface.setTransform(scale, 0, 0, scale, 0, 0);
      surface.imageSmoothingQuality = "high";
    }
    const [outgoing, incoming] = frame.layers;
    if (frame.transition && outgoing && incoming) {
      paintLayer(buffers[0].context, outgoing, frame.output);
      paintLayer(buffers[1].context, incoming, frame.output);
      context.setTransform(1, 0, 0, 1, 0, 0);
      drawTransition(context, {
        kind: frame.transition.kind,
        frames: { from: buffers[0].canvas, to: buffers[1].canvas },
        progress: frame.transition.progress,
        size,
      });
      context.setTransform(scale, 0, 0, scale, 0, 0);
    } else if (incoming ?? outgoing) {
      paintLayer(context, (incoming ?? outgoing)!, frame.output);
    } else {
      paintBackground(context, frame.output);
    }
    for (const item of frame.texts) drawText(context, item, frame.output);
  }

  return {
    draw,
    measureText: (item: VideoTextOverlay, output: VideoOutput) => textBounds(measuring, item, output),
  };
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
  if (layer.fade < 1) {
    context.fillStyle = "#000000";
    context.globalAlpha = 1 - layer.fade;
    context.fillRect(0, 0, output.width, output.height);
  }
  context.globalAlpha = 1;
}

export type Compositor = ReturnType<typeof createCompositor>;

function createBuffer() {
  const buffer = document.createElement("canvas");
  return { canvas: buffer, context: buffer.getContext("2d", { alpha: false })! };
}
