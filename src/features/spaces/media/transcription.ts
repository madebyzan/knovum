import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
export type { MediaTranscriptionProgress as SpaceMediaTranscriptionProgress } from "../contracts/generated/MediaTranscriptionProgress";

export function requestMediaTranscription(spaceId: string, nodeId: string) {
  return invoke<string>("plugin:spaces|request_media_transcription", { spaceId, nodeId });
}
