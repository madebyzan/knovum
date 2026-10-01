import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { SYSTEM_OBJECT_TYPE_IDS } from "../knowledge/model/object-types";
import { createProvenance } from "../knowledge/persistence/knowledge-operations";
import type { useSpaceStore } from "../session/space-store";

type SpaceStore = Pick<ReturnType<typeof useSpaceStore>, "activeSpace" | "addContent" | "createRelation">;

export async function createSourceEvidence(input: {
  store: SpaceStore;
  target: KnowledgePeekTarget;
  title: string;
  sourceLine: string;
  rationale: string;
}) {
  const { store, target, title, sourceLine, rationale } = input;
  const spaceId = requiredSpaceId(store);
  const quote = target.locator.quote?.exact ?? target.text ?? "";
  const [evidence] = await store.addContent({
    kind: "object",
    objectTypeId: SYSTEM_OBJECT_TYPE_IDS.evidence,
    title,
    summary: sourceLine,
    status: "accepted",
    properties: { observation: quote },
  });
  if (!evidence) return;
  await createProvenance({
    id: crypto.randomUUID(),
    spaceId,
    targetKind: "node",
    targetId: evidence.id,
    sourceNodeId: target.nodeId,
    locator: target.locator,
    sourceRevision: target.locator.sourceRevision,
    quote,
  });
  await store.createRelation(evidence.id, target.nodeId, "derives-from", { rationale, spaceId });
  return evidence;
}

function requiredSpaceId(store: SpaceStore) {
  const id = store.activeSpace?.id;
  if (!id) throw new Error("No active Space");
  return id;
}
