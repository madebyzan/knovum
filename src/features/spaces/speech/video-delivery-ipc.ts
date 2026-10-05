import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ExportVideoAudioTracksRequest } from "../contracts/generated/ExportVideoAudioTracksRequest";
import type { ExportVideoDubbingAssetRequest } from "../contracts/generated/ExportVideoDubbingAssetRequest";
import type { VideoAudioTrack } from "../contracts/generated/VideoAudioTrack";

export function listVideoAudioTracks(projectId: string) {
  return invokeCommand<VideoAudioTrack[]>("plugin:spaces|list_video_audio_tracks", { projectId });
}
export function exportVideoDubbingAsset(request: ExportVideoDubbingAssetRequest) {
  return invokeCommand<void>("plugin:spaces|export_video_dubbing_asset", { request });
}
export function exportVideoAudioTracks(request: ExportVideoAudioTracksRequest) {
  return invokeCommand<void>("plugin:spaces|export_video_audio_tracks", { request });
}
export function readDubbingSubtitles(sourcePath: string) {
  return invokeCommand<string>("plugin:spaces|read_dubbing_subtitles", { sourcePath });
}
