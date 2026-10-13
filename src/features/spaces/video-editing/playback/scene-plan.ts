import type { VideoAsset, VideoEditProject } from "../project";
import type { Scene } from "../stage/scene";
import { compositionAt, textItemsAt } from "../timing";

/** Resolves the picture an asset shows at a source position, or nothing while it is still decoding. */
export type FrameResolver = (asset: VideoAsset, sourceMs: number) => CanvasImageSource | undefined;

/**
 * Describes the frame the stage must show at one timeline position. Layers whose picture has not
 * arrived yet are left out, so the scene keeps the previous content for them instead of punching a
 * hole in the composition.
 */
export function buildScene(project: VideoEditProject, timeMs: number, resolve: FrameResolver): Scene {
  const assets = new Map(project.assets.map((asset) => [asset.nodeId, asset]));
  const composition = compositionAt(project, timeMs);
  const layers = composition.layers.flatMap((layer) => {
    const asset = assets.get(layer.clip.assetId);
    const image = asset ? resolve(asset, layer.sourceMs) : undefined;
    if (!asset || !image) return [];
    return [
      {
        id: layer.clip.id,
        image,
        asset,
        transform: layer.clip.transform,
        fade: layer.fade,
      },
    ];
  });
  return {
    output: project.output,
    layers,
    texts: textItemsAt(project, timeMs),
    transition:
      composition.transition && layers.length === 2
        ? {
            kind: composition.transition.transition.kind,
            progress: composition.transition.progress,
          }
        : undefined,
  };
}
