import { z } from "zod";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { SpaceSessionProjection } from "../contracts/generated/SpaceSessionProjection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import { relationSchema, toSpaceRelation } from "../model/relations/schema/relation";
import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { parseSpaceNodes } from "../nodes/node-operations";
import type { SpaceScene } from "../scenes/scene";
import { sceneSchema } from "../scenes/schema/scene";

const sessionSceneSchema = sceneSchema.extend({ thumbnailPath: z.string().nullable() });

export async function loadSpaceSessionProjection(spaceId: string): Promise<{
  nodes: SpaceCanvasNode[];
  scenes: SpaceScene[];
  relations: SpaceRelation[];
  editSnapshot: SpaceEditSnapshot;
}> {
  const record = await invokeCommand<SpaceSessionProjection>("plugin:spaces|load_space_session_projection", {
    spaceId,
  });
  return {
    nodes: parseSpaceNodes(record.nodes),
    scenes: record.scenes.map((scene) => sessionSceneSchema.parse(scene)),
    relations: record.relations.map((relation) => toSpaceRelation(relationSchema.parse(relation))),
    editSnapshot: parseSpaceEditSnapshot(record.editSnapshot),
  };
}
