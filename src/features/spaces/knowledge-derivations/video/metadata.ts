import { invoke } from "@tauri-apps/api/core";

import type { DerivationSource } from "../derivation-source";

export interface GeneratedVideoMetadata {
  provider: string;
  model: string;
  prompt: string;
  aspectRatio: string;
  durationSeconds: number;
  createdAt: number;
  sources: DerivationSource[];
}

export const loadGeneratedVideoMetadata = (nodeId: string) =>
  invoke<GeneratedVideoMetadata | null>("plugin:spaces|get_generated_video_metadata", { nodeId });
