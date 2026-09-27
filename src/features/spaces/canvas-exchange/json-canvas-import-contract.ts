import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/edit-snapshot";

export type JsonCanvasNodeContentV1 =
  | { type: "text"; text: string }
  | { type: "file"; file: string }
  | { type: "link"; url: string };

export interface CanvasJsonImportRequest {
  sourcePath: string;
  baseRevision: number;
}

export type CanvasJsonImportDraft = Omit<CanvasJsonImportRequest, "baseRevision">;

export interface CanvasImportReceipt {
  importedNodes: readonly { sourceId: string; nodeId: string }[];
  importedRelations: readonly SpaceRelation[];
  snapshot: SpaceEditSnapshot;
  revision: number;
}

export type CanvasImportCommit = CanvasImportReceipt;
