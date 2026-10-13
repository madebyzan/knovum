import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";
import type { CompositionLayer } from "./compositor";
import type { PreviewPlan } from "./layer-plan";
import type { MediaPool } from "./media-pool";

interface FrameResolverOptions {
  pool: MediaPool;
  materials: () => ReadonlyMap<string, VideoMaterialNode>;
  playing: () => boolean;
}

export interface ResolvedFrame {
  layers: CompositionLayer[];
  /** Every layer and audio lane the position needs is decoded and positioned. */
  complete: boolean;
  retained: Set<string>;
}

/**
 * Turns a preview plan into the media a frame can actually paint. Nothing here guesses: a layer is
 * either synchronized and decoded, or the frame is reported incomplete so the engine holds the
 * previous picture instead of showing a hole.
 */
export function createFrameResolver(options: FrameResolverOptions) {
  function resolve(plan: PreviewPlan): ResolvedFrame {
    const retained = new Set<string>();
    const layers: CompositionLayer[] = [];
    let complete = true;
    for (const layer of plan.layers) {
      const entry = admit(layer.asset.nodeId, retained, layer);
      if (!entry?.presentable) {
        complete = false;
        continue;
      }
      layers.push({
        media: entry.element as CanvasImageSource,
        asset: layer.asset,
        transform: layer.clip.transform,
        fade: layer.fade,
      });
    }
    for (const track of plan.audio) if (!admit(track.asset.nodeId, retained, track)) complete = false;
    for (const item of plan.preroll) admit(item.asset.nodeId, retained, { ...item, gain: 0 }, false);
    return { layers, complete: complete && layers.length === plan.layers.length, retained };
  }

  function admit(
    nodeId: string,
    retained: Set<string>,
    request: { asset: VideoAsset; sourceMs: number; gain: number },
    live = true,
  ) {
    const node = options.materials().get(nodeId);
    if (!node) return undefined;
    retained.add(options.pool.identity(request.asset));
    const entry = options.pool.prepare(request.asset, node);
    if (!entry) return undefined;
    const settled = options.pool.sync(entry, {
      sourceMs: request.sourceMs,
      playing: live && options.playing(),
      gain: request.gain,
    });
    return settled ? entry : undefined;
  }

  return { resolve };
}
