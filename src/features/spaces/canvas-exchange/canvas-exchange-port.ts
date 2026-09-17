import type { JsonCanvasDocumentV1 } from "./json-canvas-contract";

export interface CanvasExchangeResource {
  sourcePath: string;
  relativePath: string;
}

export interface CanvasExchangeExport {
  source: string;
  resources: readonly CanvasExchangeResource[];
}

export interface CanvasExchangeCodec {
  parse(source: string): Promise<JsonCanvasDocumentV1>;
  serialize(document: JsonCanvasDocumentV1): Promise<string>;
}

export interface CanvasExchangePort {
  importJsonCanvas(source: string, sourceDirectory: string): Promise<void>;
  exportJsonCanvas(): Promise<CanvasExchangeExport>;
}
