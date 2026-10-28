import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { ReadVideoEditAudioRequest } from "../../contracts/generated/ReadVideoEditAudioRequest";
import type { StreamVideoEditWaveformRequest } from "../../contracts/generated/StreamVideoEditWaveformRequest";
import type { VideoEditMediaSource } from "../../contracts/generated/VideoEditMediaSource";
import type { VideoEditWaveformChunk } from "../../contracts/generated/VideoEditWaveformChunk";
import type { VideoAsset } from "../project";

/** The ready revision of an asset, as every native read of its media names it. */
export function mediaSourceOf(spaceId: string, asset: VideoAsset): VideoEditMediaSource {
  return { spaceId, sourceNodeId: asset.nodeId, sourceRevision: asset.sourceRevision };
}

/** Interleaved stereo `f32le` samples of `[fromMs, fromMs + lengthMs)` of the source's audio. */
export function readSourceAudio(
  source: VideoEditMediaSource,
  range: Omit<ReadVideoEditAudioRequest, "source">,
): Promise<ArrayBuffer> {
  return invokeCommand<ArrayBuffer>("plugin:spaces|read_video_edit_audio", {
    request: { source, ...range } satisfies ReadVideoEditAudioRequest,
  });
}

/** Resolves once every peak of the source has been delivered to `onChunk`. */
export function streamSourceWaveform(
  source: VideoEditMediaSource,
  onChunk: (chunk: VideoEditWaveformChunk) => void,
): Promise<void> {
  return invokeCommand<void>("plugin:spaces|stream_video_edit_waveform", {
    request: { source } satisfies StreamVideoEditWaveformRequest,
    onPeaks: new Channel<VideoEditWaveformChunk>(onChunk),
  });
}
