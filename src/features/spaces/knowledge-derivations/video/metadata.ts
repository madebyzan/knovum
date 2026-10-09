import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { GeneratedVideoMetadata } from "../../contracts/generated/GeneratedVideoMetadata";

export function loadGeneratedVideoMetadata(nodeId: string) {
  return invokeCommand<GeneratedVideoMetadata | null>("plugin:spaces|get_generated_video_metadata", {
    nodeId,
  });
}
