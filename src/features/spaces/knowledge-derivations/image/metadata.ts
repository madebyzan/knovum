import type { VisualizationMetadata } from "../../contracts/generated/VisualizationMetadata";
export type { VisualizationMetadata } from "../../contracts/generated/VisualizationMetadata";
import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";

export const loadVisualizationMetadata = (nodeId: string) =>
  invoke<VisualizationMetadata | null>("plugin:spaces|get_visualization_metadata", { nodeId });
