import { clamp } from "es-toolkit";

import { formatBytes } from "../../../foundation/format/bytes";
import { installLocalModels, type ModelKind } from "../../../foundation/runtime/local-resources";

export interface ModelProgressInfo {
  state: "idle" | "downloading" | "verifying" | "error";
  progress: number;
  downloaded: number;
  total: number;
  partIndex?: number;
  partTotal?: number;
  error?: string;
}

export interface ModelPartDescriptor {
  kind: ModelKind;
  size: number;
}

export interface MultiPartModelDownloadOptions {
  jobId: string;
  parts: ModelPartDescriptor[];
  total: number;
  onProgress: (info: ModelProgressInfo) => void;
  onTaskProgress: (percent: number, detail: string) => void;
  isCancelled: () => boolean;
}

export function initialModelProgress(): ModelProgressInfo {
  return { state: "idle", progress: 0, downloaded: 0, total: 0 };
}

function calcProgressPercent(downloaded: number, total: number): number {
  if (total <= 0) return 0;
  return clamp(Math.round((downloaded * 100) / total), 0, 100);
}

export async function downloadMultiPartModel(options: MultiPartModelDownloadOptions) {
  const { jobId, parts, total, onProgress, onTaskProgress, isCancelled } = options;
  if (isCancelled()) throw new Error("Model download cancelled");
  await installLocalModels(
    parts.map(({ kind }) => kind),
    jobId,
    ({ downloaded }) => {
      const percent = calcProgressPercent(downloaded, total);
      onProgress({ state: "downloading", progress: percent, downloaded, total });
      onTaskProgress(percent, `${formatBytes(downloaded)} / ${formatBytes(total)}`);
    },
  );

  onProgress({
    state: "verifying",
    progress: 100,
    downloaded: total,
    total,
  });
}
