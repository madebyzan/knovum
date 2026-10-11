import { differenceBy } from "es-toolkit";

import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import { useSpaceStore } from "../session/space-store";

interface DerivationOutput {
  spaceId: string;
  nodeId: string;
  data: SpaceNodePayload;
  relations: readonly SpaceRelation[];
}

export function applyDerivationOutput(output: DerivationOutput) {
  const store = useSpaceStore();

  if (store.activeSpaceId !== output.spaceId) return false;
  store.applyNodeContentForSpace(output.spaceId, output.nodeId, output.data);
  store.relations = [...store.relations, ...differenceBy(output.relations, store.relations, ({ id }) => id)];
  store.requestReveal([output.nodeId], "ensure-visible");
  return true;
}
