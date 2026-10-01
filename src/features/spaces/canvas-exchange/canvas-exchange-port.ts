import type { JsonCanvasDocumentV1 } from "../contracts/generated/JsonCanvasDocumentV1";

export interface CanvasExchangeResource {
  sourcePath: string;
  relativePath: string;
}

export interface CanvasExchangeExport {
  document: JsonCanvasDocumentV1;
  resources: readonly CanvasExchangeResource[];
}

export interface CanvasExchangePort {
  importJsonCanvas(sourcePath: string): Promise<void>;
  exportJsonCanvas(): Promise<CanvasExchangeExport>;
}
