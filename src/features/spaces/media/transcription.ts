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

type MediaDiarizationRequest = {
  spaceId: string;
  nodeId: string;
  expectedRevision: number;
  numSpeakers: number | null;
};

export function requestMediaDiarization(request: MediaDiarizationRequest) {
  return invokeCommand<string>("plugin:spaces|request_media_diarization", request);
}
