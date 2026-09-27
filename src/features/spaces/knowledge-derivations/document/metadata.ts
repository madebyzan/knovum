import { z } from "zod";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { GroundedDocumentMetadata as WireMetadata } from "../../contracts/generated/GroundedDocumentMetadata";
import {
  groundedCitationSchema,
  type GroundedCitation,
} from "../../knowledge/model/schema/grounded-citation";

export type GroundedDocumentMetadata = Omit<WireMetadata, "citations"> & { citations: GroundedCitation[] };

export async function loadGroundedDocumentMetadata(nodeId: string): Promise<GroundedDocumentMetadata | null> {
  const value = await invokeCommand<WireMetadata | null>("plugin:spaces|get_grounded_document_metadata", {
    nodeId,
  });
  return value ? { ...value, citations: z.array(groundedCitationSchema).parse(value.citations) } : null;
}
