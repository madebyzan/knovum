import { invoke } from "@tauri-apps/api/core";

import type { DerivationSource } from "../derivation-source";

export interface VisualizationMetadata {
  provider: string;
  model: string;
  intent: string;
  style: string;
  aspectRatio: string;
  revisedPrompt: string | null;
  createdAt: number;
  sources: DerivationSource[];
}

export const loadVisualizationMetadata = (nodeId: string) =>
  invoke<VisualizationMetadata | null>("plugin:spaces|get_visualization_metadata", { nodeId });
