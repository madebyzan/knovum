import { differenceBy } from "es-toolkit";

import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { useSpaceStore } from "../session/space-store";

type Store = ReturnType<typeof useSpaceStore>;
type DerivationOutputStore = Pick<
  Store,
  "activeSpaceId" | "applyNodeContentForSpace" | "relations" | "requestReveal"
>;

interface DerivationOutput {
  spaceId: string;
  nodeId: string;
  data: SpaceNodePayload;
  relations: readonly SpaceRelation[];
}

export function applyDerivationOutput(store: DerivationOutputStore, output: DerivationOutput) {
  if (store.activeSpaceId !== output.spaceId) return false;
  store.applyNodeContentForSpace(output.spaceId, output.nodeId, output.data);
  store.relations = [...store.relations, ...differenceBy(output.relations, store.relations, ({ id }) => id)];
  store.requestReveal([output.nodeId], "ensure-visible");
  return true;
}
