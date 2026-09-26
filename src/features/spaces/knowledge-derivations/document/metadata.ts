import { z } from "zod";

import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";
import type { GroundedDocumentMetadata as WireMetadata } from "../../contracts/generated/GroundedDocumentMetadata";
import { groundedCitationSchema, type GroundedCitation } from "../../knowledge/model/grounded-citation";

export type GroundedDocumentMetadata = Omit<WireMetadata, "citations"> & { citations: GroundedCitation[] };

export async function loadGroundedDocumentMetadata(nodeId: string): Promise<GroundedDocumentMetadata | null> {
  const value = await invoke<WireMetadata | null>("plugin:spaces|get_grounded_document_metadata", { nodeId });
  return value ? { ...value, citations: z.array(groundedCitationSchema).parse(value.citations) } : null;
}
