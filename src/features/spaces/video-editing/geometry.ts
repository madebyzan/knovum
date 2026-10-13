import type { VideoAsset, VideoOutput, VideoTransform } from "./project";

export interface LayerGeometry {
  width: number;
  height: number;
  centerX: number;
  centerY: number;
}

/** Baseline ratio that fits the asset into the output frame before the clip's own scale. */
export function fitRatio(asset: VideoAsset, output: VideoOutput, fit: VideoTransform["fit"]) {
  const horizontal = output.width / asset.width;
  const vertical = output.height / asset.height;
  return fit === "contain" ? Math.min(horizontal, vertical) : Math.max(horizontal, vertical);
}

/**
 * Layer placement in output pixels. The export filter graph computes the same values, so the stage
 * preview and the rendered frame agree.
 */
export function layerGeometry(
  asset: VideoAsset,
  transform: VideoTransform,
  output: VideoOutput,
): LayerGeometry {
  const ratio = fitRatio(asset, output, transform.fit) * transform.scale;
  return {
    width: asset.width * ratio,
    height: asset.height * ratio,
    centerX: output.width / 2 + transform.x * output.width,
    centerY: output.height / 2 + transform.y * output.height,
  };
}

export function scaleFromSize(
  asset: VideoAsset,
  output: VideoOutput,
  fit: VideoTransform["fit"],
  width: number,
) {
  return width / (asset.width * fitRatio(asset, output, fit));
}
