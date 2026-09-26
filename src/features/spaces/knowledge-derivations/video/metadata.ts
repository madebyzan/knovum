import type { GeneratedVideoMetadata } from "../../contracts/generated/GeneratedVideoMetadata";
export type { GeneratedVideoMetadata } from "../../contracts/generated/GeneratedVideoMetadata";
import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";

export const loadGeneratedVideoMetadata = (nodeId: string) =>
  invoke<GeneratedVideoMetadata | null>("plugin:spaces|get_generated_video_metadata", { nodeId });
