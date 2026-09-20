import type { PackagedCanvasState } from "./core-schema";

export interface CanvasPackageRemapContext {
  nodeIds: ReadonlyMap<string, string>;
  itemIds: ReadonlyMap<string, string>;
  relationIds: ReadonlyMap<string, string>;
  assets: ReadonlyMap<string, string>;
}

export interface CanvasPackageCodec {
  validate(source: PackagedCanvasState): void;
  remap(source: PackagedCanvasState, context: CanvasPackageRemapContext): PackagedCanvasState;
}
