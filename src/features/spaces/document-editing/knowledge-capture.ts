import { addPassageNode } from "../canvas/passage-capture";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { SYSTEM_OBJECT_TYPE_IDS } from "../knowledge/model/object-types";

interface SourceEvidenceInput {
  target: KnowledgePeekTarget;
  title: string;
  sourceLine: string;
  rationale?: string | undefined;
}

export async function createSourceEvidence(input: SourceEvidenceInput) {
  const { target, title, sourceLine } = input;
  const quote = target.locator.quote?.exact ?? target.text ?? "";
  return addPassageNode(
    {
      kind: "object",
      objectTypeId: SYSTEM_OBJECT_TYPE_IDS.evidence,
      title,
      summary: sourceLine,
      status: "accepted",
      properties: { observation: quote },
    },
    [{ nodeId: target.nodeId, locator: target.locator, quote }],
  );
}
