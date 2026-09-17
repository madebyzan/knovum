import { uniqBy } from "es-toolkit";

import type { KnowledgeChange } from "../knowledge/model/change-set";
import type { GroundedCitation } from "../knowledge/model/grounded-citation";
import type { GroundedMessage } from "../knowledge/model/grounded-message";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { SYSTEM_OBJECT_TYPE_IDS } from "../knowledge/model/object-types";
import { commitKnowledgeChanges, createProvenance } from "../knowledge/persistence/knowledge-repository";
import type { useSpaceStore } from "../session/space-store";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export async function commitGroundedAnswerKnowledge(
  store: SpaceStore,
  answer: GroundedMessage,
  title: string,
  rationale: string,
) {
  const spaceId = requiredSpaceId(store);
  const claimId = "claim";
  const operations: KnowledgeChange[] = [
    {
      op: "create-object",
      tempId: claimId,
      objectTypeId: SYSTEM_OBJECT_TYPE_IDS.claim,
      title,
      summary: answer.content,
      status: "draft",
      properties: { statement: answer.content },
    },
  ];
  uniqueCurrentCitations(answer).forEach((citation, index) => {
    const evidenceId = `evidence-${index}`;
    operations.push(
      {
        op: "create-object",
        tempId: evidenceId,
        objectTypeId: SYSTEM_OBJECT_TYPE_IDS.evidence,
        title: citation.nodeTitle,
        summary: citation.quote,
        status: "draft",
        properties: { observation: citation.quote },
      },
      {
        op: "add-provenance",
        targetKind: "node",
        targetId: evidenceId,
        sourceNodeId: citation.nodeId,
        locator: citation.locator,
        sourceRevision: citation.sourceRevision,
        quote: citation.quote,
      },
      {
        op: "create-relation",
        tempId: `supports-${index}`,
        sourceId: evidenceId,
        targetId: claimId,
        kind: "supports",
        rationale,
      },
    );
  });
  return commitKnowledgeChanges({
    spaceId,
    actor: answer.model ?? answer.provider ?? "knovum-grounded-assistant",
    operations,
  });
}

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
  const [evidence] = await store.addItem({
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

function uniqueCurrentCitations(answer: GroundedMessage) {
  const current = answer.citations.filter(
    (citation): citation is Extract<GroundedCitation, { state: "current" }> => citation.state === "current",
  );
  return uniqBy(current, (citation) => citation.chunkId);
}

function requiredSpaceId(store: SpaceStore) {
  const id = store.activeSpace?.id;
  if (!id) throw new Error("No active Space");
  return id;
}
