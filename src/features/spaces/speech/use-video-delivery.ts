import { save } from "@tauri-apps/plugin-dialog";
import { onScopeDispose, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { exportFilter, safeExportFileName } from "../assets/export/naming";
import type { ExportVideoAudioTracksRequest } from "../contracts/generated/ExportVideoAudioTracksRequest";
import type { ExportVideoDubbingAssetRequest } from "../contracts/generated/ExportVideoDubbingAssetRequest";
import type { VideoDeliveryAsset } from "../contracts/generated/VideoDeliveryAsset";
import { cancelSpeechGeneration } from "./ipc";
import { exportVideoDubbingAsset, exportVideoAudioTracks } from "./video-delivery-ipc";

export function useVideoDelivery(project: () => { projectId: string; revisionId: string; title: string }) {
  const busy = shallowRef(false);
  const error = shallowRef<string>();
  const activeJobId = shallowRef<string>();
  let epoch = 0;
  async function cancel() {
    if (activeJobId.value) await cancelSpeechGeneration(activeJobId.value);
  }
  function reset() {
    epoch += 1;
    void cancel().catch((cause) => {
      error.value = getErrorMessage(cause);
    });
  }
  watch(() => project().revisionId, reset);
  onScopeDispose(reset);
  async function exportFile(
    extension: string,
    suffix: string,
    operation: (jobId: string, destinationPath: string) => Promise<void>,
  ) {
    if (busy.value) return;
    busy.value = true;
    const version = epoch;
    const job = crypto.randomUUID();
    error.value = undefined;
    try {
      const destinationPath = await save({
        defaultPath: `${safeExportFileName(project().title)}-${suffix}.${extension}`,
        filters: [exportFilter(extension)],
      });
      if (!destinationPath || epoch !== version) return;
      activeJobId.value = job;
      await operation(job, destinationPath);
    } catch (cause) {
      if (epoch === version) error.value = getErrorMessage(cause);
    } finally {
      if (activeJobId.value === job) activeJobId.value = undefined;
      busy.value = false;
    }
  }
  function exportAsset(asset: VideoDeliveryAsset) {
    const owner = project();
    return exportFile(asset === "srt" || asset === "vtt" ? asset : "wav", asset, (jobId, destinationPath) => {
      const request: ExportVideoDubbingAssetRequest = {
        projectId: owner.projectId,
        revisionId: owner.revisionId,
        jobId,
        destinationPath,
        asset,
      };
      return exportVideoDubbingAsset(request);
    });
  }
  function exportTracks(revisionIds: string[], retainOriginalTrack: boolean) {
    const projectId = project().projectId;
    return exportFile("mp4", "tracks", (jobId, destinationPath) => {
      const request: ExportVideoAudioTracksRequest = {
        projectId,
        revisionIds,
        retainOriginalTrack,
        jobId,
        destinationPath,
      };
      return exportVideoAudioTracks(request);
    });
  }
  return { busy, error, cancel, exportAsset, exportTracks };
}
