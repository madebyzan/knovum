import { info as logInfo } from "@tauri-apps/plugin-log";

import { createAppError, decodeAppError } from "../../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { i18n } from "../../../../foundation/i18n/instance";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { formatByteSize } from "../../assets/format-byte-size";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import { nodeTaskSubject } from "../../nodes/node-task-subject";
import { cancelMediaImport, requestMediaImport } from "./ipc";
import type { ImportedMedia, MediaImportProgress, ResolvedMedia } from "./schema/import";

export type MediaSourceAcquisition =
  | { outcome: "imported"; payload: SpaceMediaPayload }
  | { outcome: "unsupported" };

/** Whoever asked for the media decides what to do the moment the link is identified. */
export interface MediaAcquisitionObserver {
  /**
   * Called once, before the first byte, with what the link turned out to be. The canvas reserves
   * the node for the media here, so the work has somewhere to show itself and someone to belong
   * to; the node it returns becomes the subject of this acquisition's task.
   */
  resolved: (media: ResolvedMedia) => Promise<{ nodeId: string; title: string }>;
}

export async function acquireMediaSource(
  url: string,
  observer: MediaAcquisitionObserver,
): Promise<MediaSourceAcquisition> {
  const tasks = useTaskStore();
  const jobId = crypto.randomUUID();
  let loggedStage: MediaImportProgress["stage"] | undefined;
  let reservation: Promise<{ nodeId: string; title: string }> | undefined;
  tasks.restoreRunning(
    {
      id: jobId,
      title: i18n.global.t("spaces.media.import.title"),
      statusMessage: i18n.global.t("spaces.media.import.resolving"),
      startedAt: Date.now(),
    },
    () => cancelMediaImport(jobId),
  );
  try {
    void logInfo("[spaces.url-capture] media acquisition started");
    const mediaAcquisition = await requestMediaImport({
      jobId,
      resourceTitle: i18n.global.t("spaces.media.import.preparing"),
      url,
      onEvent: (event) => {
        if (event.event === "resolved") {
          void logInfo(`[spaces.url-capture] media resolved; kind=${event.kind}`);
          reservation ??= reserve(jobId, event, observer);
          return;
        }
        if (event.stage !== loggedStage) {
          loggedStage = event.stage;
          void logInfo(`[spaces.url-capture] media stage=${event.stage}`);
        }
        updateProgress(jobId, event);
      },
    });
    if (mediaAcquisition.outcome === "unsupported") {
      void logInfo("[spaces.url-capture] media source unsupported");
      tasks.dismiss(jobId);
      return mediaAcquisition;
    }
    // The engine names the source before it downloads it, so an import that never named one has
    // nowhere to land.
    if (!reservation) throw new Error("The media source was never identified");
    await reservation;
    tasks.succeed(jobId);
    void logInfo("[spaces.url-capture] media acquisition completed");
    return { outcome: "imported", payload: mediaPayloadFromImport(mediaAcquisition.media) };
  } catch (error) {
    if (decodeAppError(error)?.code === "media_acquisition_cancelled") {
      tasks.dismiss(jobId);
      throw createAppError("media_acquisition_cancelled", "Media acquisition cancelled");
    }
    tasks.fail(jobId, getErrorMessage(error));
    throw error;
  }
}

async function reserve(jobId: string, media: ResolvedMedia, observer: MediaAcquisitionObserver) {
  const tasks = useTaskStore();
  const reserved = await observer.resolved(media);
  tasks.title(jobId, reserved.title);
  tasks.describeSubject(jobId, nodeTaskSubject(reserved.nodeId, media.kind, reserved.title));
  return reserved;
}

function mediaPayloadFromImport(imported: ImportedMedia): SpaceMediaPayload {
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
  const statusMessage = i18n.global.t(`spaces.media.import.${progress.stage}`);
  const finalizingPercent = finalizingProgressPercent(progress);
  if (finalizingPercent !== null) {
    tasks.progress(taskId, finalizingPercent, statusMessage);
    return;
  }
  if (progress.stage !== "downloading" && progress.stage !== "preparing") {
    tasks.setStatusMessage(taskId, statusMessage);
    return;
  }
  const downloaded = formatByteSize(progress.downloadedBytes) ?? "";
  if (progress.totalBytes === null || progress.totalBytes === 0) {
    tasks.setStatusMessage(taskId, downloaded ? `${statusMessage} · ${downloaded}` : statusMessage);
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
