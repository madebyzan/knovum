import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { LocalModelId } from "../../../shared/contracts/generated/LocalModelId";

export function requestMediaTranscription(spaceId: string, nodeId: string, model?: LocalModelId) {
  return invokeCommand<string>("plugin:spaces|request_media_transcription", { spaceId, nodeId, model });
}

type MediaAlignmentRequest = {
  spaceId: string;
  nodeId: string;
  expectedRevision: number;
  segmentIds: string[];
  model?: LocalModelId | undefined;
};

export function requestMediaAlignment(request: MediaAlignmentRequest) {
  return invokeCommand<string>("plugin:spaces|request_media_alignment", request);
}

type MediaDiarizationRequest = {
  spaceId: string;
  nodeId: string;
  expectedRevision: number;
  numSpeakers: number | null;
  model?: LocalModelId | undefined;
};

export function requestMediaDiarization(request: MediaDiarizationRequest) {
  return invokeCommand<string>("plugin:spaces|request_media_diarization", request);
}
