import { invoke } from "@tauri-apps/api/core";

import type { GroundedCitation } from "../../knowledge/model/grounded-citation";
import type { DerivationSource } from "../derivation-source";

export interface GroundedDocumentMetadata {
  actionKind: "draft-document";
  provider: string;
  model: string;
  createdAt: number;
  citations: GroundedCitation[];
  sources: DerivationSource[];
}

export function loadGroundedDocumentMetadata(nodeId: string) {
  return invoke<GroundedDocumentMetadata | null>("plugin:spaces|get_grounded_document_metadata", { nodeId });
}
