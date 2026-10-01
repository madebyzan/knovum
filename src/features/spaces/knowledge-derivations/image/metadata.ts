import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { VisualizationMetadata } from "../../contracts/generated/VisualizationMetadata";

export const loadVisualizationMetadata = (nodeId: string) =>
  invokeCommand<VisualizationMetadata | null>("plugin:spaces|get_visualization_metadata", { nodeId });
