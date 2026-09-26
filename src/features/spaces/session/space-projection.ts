import { z } from "zod";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceSessionProjection } from "../contracts/generated/SpaceSessionProjection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import { parseSpaceNodeRecords } from "../nodes/node-operations";
import { relationSchema, sceneSchema, toSpaceRelation } from "../package/core-schema";
import type { SpaceScene } from "../scenes/scene";

const sessionSceneSchema = sceneSchema.extend({ thumbnailPath: z.string().nullable() });

export async function loadSpaceSessionProjection(spaceId: string): Promise<{
  nodes: SpaceCanvasNode[];
  scenes: SpaceScene[];
  relations: SpaceRelation[];
  editSnapshot: SpaceEditSnapshot;
}> {
  const record = await invoke<SpaceSessionProjection>("plugin:spaces|load_space_session_projection", {
    spaceId,
  });
  return {
    nodes: parseSpaceNodeRecords(record.nodes),
    scenes: record.scenes.map((scene) => sessionSceneSchema.parse(scene)),
    relations: record.relations.map((relation) => toSpaceRelation(relationSchema.parse(relation))),
    editSnapshot: parseSpaceEditSnapshot(record.editSnapshot),
  };
}
