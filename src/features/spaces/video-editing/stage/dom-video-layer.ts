import { layerGeometry } from "../geometry";
import type { VideoOutput } from "../project";
import type { Scene, SceneLayer } from "./scene";
import { PLAIN_LAYER, transitionStyle, type LayerStyle } from "./scene-transition";

interface DomLayerSlot {
  wrapper: HTMLDivElement;
  mediaFrame: HTMLDivElement;
  curtain: HTMLDivElement;
  canvas?: HTMLCanvasElement | undefined;
  lastBitmap?: CanvasImageSource | undefined;
  lastMedia?: HTMLVideoElement | undefined;
  lastLayoutKey?: string | undefined;
}

function clipPathOf(style: LayerStyle, output: VideoOutput): string {
  if (!style.clip) return "none";
  const top = (style.clip.y / output.height) * 100;
  const right = ((output.width - (style.clip.x + style.clip.width)) / output.width) * 100;
  const bottom = ((output.height - (style.clip.y + style.clip.height)) / output.height) * 100;
  const left = (style.clip.x / output.width) * 100;
  return `inset(${top}% ${right}% ${bottom}% ${left}%)`;
}

function createSlot(): DomLayerSlot {
  const wrapper = document.createElement("div");
  wrapper.style.cssText =
    "position:absolute;inset:0;overflow:hidden;pointer-events:none;will-change:transform,opacity;";
  const mediaFrame = document.createElement("div");
  mediaFrame.style.cssText = "position:absolute;transform-origin:center center;pointer-events:none;";
  const curtain = document.createElement("div");
  curtain.style.cssText = "position:absolute;inset:0;background:#000;pointer-events:none;";
  wrapper.append(mediaFrame, curtain);
  return { wrapper, mediaFrame, curtain };
}

function attachVideo(slot: DomLayerSlot, video: HTMLVideoElement) {
  if (slot.canvas) {
    slot.canvas.remove();
    slot.canvas = undefined;
    slot.lastBitmap = undefined;
  }
  if (slot.lastMedia !== video || video.parentElement !== slot.mediaFrame) {
    slot.lastMedia?.remove();
    video.style.cssText = "width:100%;height:100%;object-fit:fill;display:block;pointer-events:none;";
    slot.mediaFrame.appendChild(video);
    slot.lastMedia = video;
  }
}

function attachStill(slot: DomLayerSlot, image: CanvasImageSource) {
  if (slot.lastMedia) {
    slot.lastMedia.remove();
    slot.lastMedia = undefined;
  }
  if (!slot.canvas) {
    slot.canvas = document.createElement("canvas");
    slot.canvas.style.cssText = "width:100%;height:100%;display:block;pointer-events:none;";
    slot.mediaFrame.appendChild(slot.canvas);
  }
  if (slot.lastBitmap === image) return;
  slot.lastBitmap = image;
  const width = "width" in image && typeof image.width === "number" ? image.width : 1;
  const height = "height" in image && typeof image.height === "number" ? image.height : 1;
  slot.canvas.width = Math.max(1, width);
  slot.canvas.height = Math.max(1, height);
  slot.canvas.getContext("2d")?.drawImage(image, 0, 0);
}

function attachMedia(slot: DomLayerSlot, image: CanvasImageSource) {
  if (typeof HTMLVideoElement !== "undefined" && image instanceof HTMLVideoElement) {
    attachVideo(slot, image);
    return;
  }
  attachStill(slot, image);
}

function slotLayoutKey(
  layer: SceneLayer,
  frame: { output: VideoOutput; scale: number; style: LayerStyle; zIndex: number },
): string {
  const { output, scale, style, zIndex } = frame;
  const t = layer.transform;
  const c = style.clip;
  return `${zIndex}|${scale}|${output.width}x${output.height}|${output.background}|${style.opacity}|${style.dx}|${style.dy}|${c?.x ?? ""}|${c?.y ?? ""}|${c?.width ?? ""}|${c?.height ?? ""}|${t.x}|${t.y}|${t.scale}|${t.rotation}|${t.opacity}|${t.fit}|${t.flipHorizontal}|${t.flipVertical}|${layer.fade}|${layer.asset.width}x${layer.asset.height}`;
}

function applySlotStyle(
  slot: DomLayerSlot,
  layer: SceneLayer,
  frame: { output: VideoOutput; scale: number; style: LayerStyle; zIndex: number },
) {
  const key = slotLayoutKey(layer, frame);
  if (slot.lastLayoutKey === key) return;
  slot.lastLayoutKey = key;
  const { output, scale, style, zIndex } = frame;
  slot.wrapper.style.zIndex = String(zIndex);
  slot.wrapper.style.backgroundColor = output.background;
  slot.wrapper.style.opacity = String(style.opacity);
  slot.wrapper.style.transform =
    style.dx === 0 && style.dy === 0
      ? "none"
      : `translate3d(${style.dx * scale}px, ${style.dy * scale}px, 0)`;
  slot.wrapper.style.clipPath = clipPathOf(style, output);

  const geometry = layerGeometry(layer.asset, layer.transform, output);
  const flipX = layer.transform.flipHorizontal ? -1 : 1;
  const flipY = layer.transform.flipVertical ? -1 : 1;
  slot.mediaFrame.style.left = `${geometry.centerX * scale}px`;
  slot.mediaFrame.style.top = `${geometry.centerY * scale}px`;
  slot.mediaFrame.style.width = `${geometry.width * scale}px`;
  slot.mediaFrame.style.height = `${geometry.height * scale}px`;
  slot.mediaFrame.style.opacity = String(layer.transform.opacity);
  slot.mediaFrame.style.transform = `translate(-50%, -50%) rotate(${layer.transform.rotation}deg) scale(${flipX}, ${flipY})`;
  slot.curtain.style.opacity = String(1 - layer.fade);
}

/**
 * Hardware-composited DOM media surface placed directly behind the Konva interaction and caption
 * layer. Native `<video>` elements run straight through AVFoundation / VideoToolbox / Metal with
 * zero 2D canvas pixel readbacks, sustaining full source frame rates (24–60 fps).
 */
export function createDomVideoLayer(container: HTMLDivElement) {
  const root = document.createElement("div");
  root.style.cssText = "position:absolute;inset:0;overflow:hidden;pointer-events:none;";
  container.appendChild(root);
  const slots = new Map<string, DomLayerSlot>();

  return {
    present(scene: Scene, scale: number) {
      const transition = scene.transition ? transitionStyle(scene.transition, scene.output) : undefined;
      root.style.backgroundColor = transition?.backdrop ?? scene.output.background;
      const activeIds = new Set<string>();
      for (const [index, layer] of scene.layers.entries()) {
        activeIds.add(layer.id);
        let slot = slots.get(layer.id);
        if (!slot) {
          slot = createSlot();
          slots.set(layer.id, slot);
          root.appendChild(slot.wrapper);
        }
        attachMedia(slot, layer.image);
        const style = transition ? (index === 0 ? transition.from : transition.to) : PLAIN_LAYER;
        applySlotStyle(slot, layer, { output: scene.output, scale, style, zIndex: index + 1 });
      }
      for (const [id, slot] of slots) {
        if (activeIds.has(id)) continue;
        slot.wrapper.remove();
        slots.delete(id);
      }
    },
    dispose() {
      for (const slot of slots.values()) slot.wrapper.remove();
      slots.clear();
      root.remove();
    },
  };
}
