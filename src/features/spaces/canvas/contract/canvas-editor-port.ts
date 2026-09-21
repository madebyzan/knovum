import type {
  CanvasDocumentV1,
  CanvasNodeAppearance,
  CanvasNodeSizeAuthority,
  CanvasRelationColor,
} from "./canvas-document";

export interface CanvasPoint {
  x: number;
  y: number;
}

export interface CanvasViewport extends CanvasPoint {
  zoom: number;
}

export interface CanvasNodePlacement extends CanvasPoint {
  id: string;
}

export interface CanvasSelectionSnapshot {
  nodeIds: readonly string[];
  structureNodeIds: readonly string[];
  nodePlacements: readonly CanvasNodePlacement[];
  relationIds: readonly string[];
  singleNodeAppearance?: CanvasNodeAppearance;
  bounds?: CanvasRect;
}

export type CanvasLayerMove = "front" | "forward" | "backward" | "back";
export type CanvasAlignMode = "left" | "center" | "right" | "top" | "middle" | "bottom";
export type CanvasDistributeMode = "horizontal" | "vertical";
export type CanvasAutoLayoutDirection = "RIGHT" | "DOWN";

export interface CanvasRect extends CanvasPoint {
  width: number;
  height: number;
}

export interface CanvasContentHeightMeasurement {
  contentRevision: string | null;
  height: number;
  sizeAuthority: CanvasNodeSizeAuthority;
  typographyScale: number;
  width: number;
}

export interface CanvasEditorPort {
  cancelInteraction(): void;
  copySelection(): Promise<void>;
  cutSelection(): Promise<void>;
  pasteSelection(): Promise<boolean>;
  deleteSelection(): Promise<void>;
  duplicateSelection(): Promise<void>;
  nudgeSelection(offset: CanvasPoint): void;
  moveSelectionLayer(mode: CanvasLayerMove): void;
  alignSelection(mode: CanvasAlignMode): Promise<void>;
  autoLayoutSelection(direction: CanvasAutoLayoutDirection): Promise<void>;
  distributeSelection(mode: CanvasDistributeMode): Promise<void>;
  selectAll(): void;
  zoomIn(): void;
  zoomOut(): void;
  setZoom(zoom: number): void;
  reveal(nodeIds: readonly string[], mode: "ensure-visible" | "focus"): void;
  navigateViewport(viewport: CanvasViewport): void;
  setViewport(viewport: CanvasViewport): void;
  setRelationColor(relationId: string, color: CanvasRelationColor): void;
  setNodeAppearance(nodeId: string, patch: Partial<CanvasNodeAppearance>): void;
  fitNodeContentHeight(nodeId: string, measurement: CanvasContentHeightMeasurement): void;
  flushContentHeightFitting(): boolean;
  setContentHeightFittingPaused(paused: boolean): void;
  getViewport(): CanvasViewport;
  getDocument(): CanvasDocumentV1;
  observeDocument(observer: (document: CanvasDocumentV1) => void): () => void;
  adoptDocument(document: CanvasDocumentV1): void;
  zoomToFit(): void;
  zoomToSelection(): void;
  clearSelection(): void;
  captureSceneViewport(): Promise<Blob>;
  captureCanvasOverview(): Promise<Blob>;
  getSelectionSnapshot(): CanvasSelectionSnapshot;
  getSelectionBounds(): CanvasRect | undefined;
  selectNodes(nodeIds: readonly string[]): void;
  selectStructureNodes(structureNodeIds: readonly string[]): void;
  screenToCanvasPoint(point: CanvasPoint): CanvasPoint;
  canvasToViewportPoint(point: CanvasPoint): CanvasPoint;
}
