import { invokeCommand } from "../../../foundation/ipc/invoke-command";

export function requestMediaTranscription(spaceId: string, nodeId: string) {
  return invokeCommand<string>("plugin:spaces|request_media_transcription", { spaceId, nodeId });
}
