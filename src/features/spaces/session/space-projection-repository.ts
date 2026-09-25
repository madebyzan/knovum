import { invoke } from "@tauri-apps/api/core";

import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import { parseSpaceNodeRecords, type SpaceNodeRecord } from "../nodes/node-repository";
import type { SpaceScene } from "../scenes/scene";

interface SpaceSessionProjectionRecord {
  nodes: SpaceNodeRecord[];
  scenes: SpaceScene[];
  relations: SpaceRelation[];
  editSnapshot: unknown;
}

export async function loadSpaceSessionProjection(spaceId: string): Promise<{
  nodes: SpaceCanvasNode[];
  scenes: SpaceScene[];
  relations: SpaceRelation[];
  editSnapshot: SpaceEditSnapshot;
}> {
  const record = await invoke<SpaceSessionProjectionRecord>("plugin:spaces|load_space_session_projection", {
    spaceId,
  });
  return {
    nodes: parseSpaceNodeRecords(record.nodes),
    scenes: record.scenes,
    relations: record.relations,
    editSnapshot: parseSpaceEditSnapshot(record.editSnapshot),
  };
}
