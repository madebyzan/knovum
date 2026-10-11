import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { SYSTEM_OBJECT_TYPE_IDS } from "../knowledge/model/object-types";
import { attachProvenanceFromPeek } from "../knowledge/provenance-actions";
import { useSpaceStore } from "../session/space-store";

interface SourceEvidenceInput {
  target: KnowledgePeekTarget;
  title: string;
  sourceLine: string;
  rationale: string;
}

export async function createSourceEvidence(input: SourceEvidenceInput) {
  const store = useSpaceStore();

  const { target, title, sourceLine, rationale } = input;
  const spaceId = store.activeSpace?.id;
  if (!spaceId) throw new Error("No active Space");
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
  await attachProvenanceFromPeek(
    {
      spaceId,
      targetKind: "node",
      targetId: evidence.id,
    },
    target,
  );
  await store.createRelation(evidence.id, target.nodeId, "derives-from", { rationale, spaceId });
  return evidence;
}
