import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { useSpaceStore } from "../session/space-store";

type Store = ReturnType<typeof useSpaceStore>;
type DerivationOutputStore = Pick<
  Store,
  "activeSpaceId" | "applyNodeDataForSpace" | "relations" | "requestReveal"
>;

interface DerivationOutput {
  spaceId: string;
  nodeId: string;
  data: SpaceNodePayload;
  relations: readonly SpaceRelation[];
}

export function applyDerivationOutput(store: DerivationOutputStore, output: DerivationOutput) {
  if (store.activeSpaceId !== output.spaceId) return false;
  store.applyNodeDataForSpace(output.spaceId, output.nodeId, output.data);
  const ids = new Set(store.relations.map(({ id }) => id));
  store.relations = [...store.relations, ...output.relations.filter(({ id }) => !ids.has(id))];
  store.requestReveal([output.nodeId], "ensure-visible");
  return true;
}
