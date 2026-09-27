import { info as logInfo } from "@tauri-apps/plugin-log";

import { i18n } from "../../../../foundation/i18n/i18n";
import { decodeCommandError } from "../../../../foundation/ipc/command-error";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { formatByteSize } from "../../assets/format-byte-size";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import type { ImportedMedia, MediaImportProgress } from "./contract";
import { cancelMediaImport, requestMediaImport } from "./ipc";

export class MediaImportCancelledError extends Error {}

export type MediaSourceAcquisition =
  | { outcome: "imported"; payload: SpaceMediaPayload }
  | { outcome: "unsupported" };

export async function acquireMediaSource(
  url: string,
  onProgress: (progress: MediaImportProgress) => void,
): Promise<MediaSourceAcquisition> {
  const tasks = useTaskStore();
  const jobId = crypto.randomUUID();
  let cancelRequested = false;
  let loggedStage: MediaImportProgress["stage"] | undefined;
  tasks.start(
    i18n.global.t("spaces.media.import.title"),
    i18n.global.t("spaces.media.import.resolving"),
    () => {
      cancelRequested = true;
      return cancelMediaImport(jobId);
    },
    jobId,
  );
  try {
    void logInfo("[spaces.url-capture] media acquisition started");
    const mediaAcquisition = await requestMediaImport(
      jobId,
      i18n.global.t("spaces.media.import.preparing"),
      url,
      (progress) => {
        if (progress.stage !== loggedStage) {
          loggedStage = progress.stage;
          void logInfo(`[spaces.url-capture] media stage=${progress.stage}`);
        }
        updateProgress(jobId, progress);
        onProgress(progress);
      },
    );
    if (mediaAcquisition.outcome === "unsupported") {
      void logInfo("[spaces.url-capture] media source unsupported");
      tasks.dismiss(jobId);
      return mediaAcquisition;
    }
    tasks.title(jobId, mediaAcquisition.media.title);
    tasks.succeed(jobId);
    void logInfo("[spaces.url-capture] media acquisition completed");
    return { outcome: "imported", payload: mediaPayloadFromImport(mediaAcquisition.media) };
  } catch (error) {
    if (cancelRequested || decodeCommandError(error)?.code === "media_acquisition_cancelled") {
      tasks.dismiss(jobId);
      throw new MediaImportCancelledError();
    }
    tasks.fail(jobId, String(error));
    throw error;
  }
}

export function mediaPayloadFromImport(imported: ImportedMedia): SpaceMediaPayload {
  const common = {
    title: imported.title,
    assetPath: imported.assetPath,
    fileName: imported.fileName,
    size: imported.size,
    modifiedAt: imported.modifiedAt,
    sourceRevision: imported.sourceRevision,
    origin: imported.origin,
    ...(imported.creator === null ? {} : { creator: imported.creator }),
    ...(imported.durationSeconds === null ? {} : { durationSeconds: imported.durationSeconds }),
    ...(imported.previewPath === null ? {} : { previewPath: imported.previewPath }),
  };
  if (imported.kind === "audio") return { kind: "audio", ...common };
  return {
    kind: "video",
    ...common,
    ...(imported.intrinsicWidth === null ? {} : { intrinsicWidth: imported.intrinsicWidth }),
    ...(imported.intrinsicHeight === null ? {} : { intrinsicHeight: imported.intrinsicHeight }),
  };
}

function updateProgress(taskId: string, progress: MediaImportProgress) {
  const tasks = useTaskStore();
  const detail = i18n.global.t(`spaces.media.import.${progress.stage}`);
  const finalizingPercent = finalizingProgressPercent(progress);
  if (finalizingPercent !== null) {
    tasks.progress(taskId, finalizingPercent, detail);
    return;
  }
  if (progress.stage !== "downloading" && progress.stage !== "preparing") {
    tasks.detail(taskId, detail);
    return;
  }
  const downloaded = formatByteSize(progress.downloadedBytes) ?? "";
  if (progress.totalBytes === null || progress.totalBytes === 0) {
    tasks.detail(taskId, downloaded ? `${detail} · ${downloaded}` : detail);
    return;
  }
  const total = formatByteSize(progress.totalBytes) ?? "";
  tasks.progress(taskId, (progress.downloadedBytes / progress.totalBytes) * 100, `${downloaded} / ${total}`);
}

function finalizingProgressPercent(progress: MediaImportProgress): number | null {
  if (
    progress.stage !== "finalizing" ||
    progress.processedMs === null ||
    progress.durationMs === null ||
    progress.durationMs <= 0
  )
    return null;
  return Math.min(100, (progress.processedMs / progress.durationMs) * 100);
}
