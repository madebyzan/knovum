import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { VisualizationMetadata } from "../../contracts/generated/VisualizationMetadata";

export function loadVisualizationMetadata(nodeId: string) {
  return invokeCommand<VisualizationMetadata | null>("plugin:spaces|get_visualization_metadata", { nodeId });
}
