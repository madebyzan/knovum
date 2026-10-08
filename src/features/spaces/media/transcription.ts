import { invokeCommand } from "../../../foundation/ipc/invoke-command";

export function requestMediaTranscription(spaceId: string, nodeId: string) {
  return invokeCommand<string>("plugin:spaces|request_media_transcription", { spaceId, nodeId });
}

export function requestMediaAlignment(
  spaceId: string,
  nodeId: string,
  expectedRevision: number,
  segmentIds: string[],
) {
  return invokeCommand<string>("plugin:spaces|request_media_alignment", {
    spaceId,
    nodeId,
    expectedRevision,
    segmentIds,
  });
}

export function requestMediaDiarization(request: {
  spaceId: string;
  nodeId: string;
  expectedRevision: number;
  numSpeakers: number | null;
}) {
  return invokeCommand<string>("plugin:spaces|request_media_diarization", request);
}
