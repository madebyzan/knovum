import type { GeneratedVideoMetadata } from "../../contracts/generated/GeneratedVideoMetadata";
export type { GeneratedVideoMetadata } from "../../contracts/generated/GeneratedVideoMetadata";
import { invokeCommand } from "../../../../foundation/ipc/invoke-command";

export const loadGeneratedVideoMetadata = (nodeId: string) =>
  invokeCommand<GeneratedVideoMetadata | null>("plugin:spaces|get_generated_video_metadata", { nodeId });
