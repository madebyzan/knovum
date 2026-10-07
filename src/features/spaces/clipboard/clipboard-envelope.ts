import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { StructureLayout, StructureNode } from "../model/structure/schema/structure";

interface SpaceClipboardNode {
  id: string;
  data: SpaceNodePayload;
}

export interface SpaceClipboardPayload {
  version: 1;
  structure: StructureNode[];
  layouts: StructureLayout[];
  nodes: SpaceClipboardNode[];
  relations: SpaceRelation[];
}
