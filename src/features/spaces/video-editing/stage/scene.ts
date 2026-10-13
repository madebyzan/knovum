import type {
  VideoAsset,
  VideoOutput,
  VideoTextOverlay,
  VideoTransform,
  VideoTransitionKind,
} from "../project";

export interface SceneLayer {
  /** Clip identity; the scene reuses one node per clip across frames. */
  id: string;
  image: CanvasImageSource;
  asset: VideoAsset;
  transform: VideoTransform;
  /** Visual fade gain; the layer dips to the same black the export fades to. */
  fade: number;
}

export interface Scene {
  output: VideoOutput;
  layers: SceneLayer[];
  texts: VideoTextOverlay[];
  transition?: { kind: VideoTransitionKind; progress: number } | undefined;
}
