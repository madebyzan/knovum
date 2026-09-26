import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import {
  cancelModelInstall,
  getLocalResourcesStatus,
  removeLocalModel,
  textEmbeddingReady,
  transcriptionReady,
  type ModelKind,
  type LocalResourcesStatus,
} from "../../../foundation/runtime/local-resources";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { downloadMultiPartModel, initialModelProgress, type ModelProgressInfo } from "./model-download";

export type { ModelProgressInfo };

type RuntimeActionKind = "transcription" | "textEmbedding" | "refresh";
type RuntimeDownloadState = { jobId: string; taskId?: string; cancelled: boolean };

const TRANSCRIPTION_KINDS: ModelKind[] = ["transcription", "transcriptionVad"];
const TEXT_EMBEDDING_KINDS: ModelKind[] = [
  "textEmbeddingConfig",
  "textEmbeddingTokenizer",
  "textEmbeddingWeights",
];

export function useRuntimeSettings() {
  const { t } = useI18n();
  const tasks = useTaskStore();
  const status = ref<LocalResourcesStatus>();
  const busy = ref<RuntimeActionKind>();
  const activeDownload = ref<RuntimeDownloadState>();

  const transcriptionProgress = ref<ModelProgressInfo>(initialModelProgress());
  const textEmbeddingProgress = ref<ModelProgressInfo>(initialModelProgress());

  const transcriptionInstalled = computed(() => transcriptionReady(status.value));
  const textEmbeddingInstalled = computed(() => textEmbeddingReady(status.value));

  const modelStats = (kinds: ModelKind[]) => ({
    count: computed(() => {
      const current = status.value;
      return current ? kinds.reduce((count, kind) => count + Number(current.models[kind].verified), 0) : 0;
    }),
    totalCount: kinds.length,
    size: computed(() => {
      const current = status.value;
      return current ? kinds.reduce((bytes, kind) => bytes + current.models[kind].downloadSize, 0) : 0;
    }),
  });

  const transcriptionStats = modelStats(TRANSCRIPTION_KINDS);
  const textEmbeddingStats = modelStats(TEXT_EMBEDDING_KINDS);

  async function refresh() {
    busy.value = "refresh";
    try {
      status.value = await getLocalResourcesStatus();
    } catch (error) {
      toast.error(String(error));
    } finally {
      busy.value = undefined;
    }
  }

  async function download(
    kind: RuntimeActionKind,
    title: string,
    kinds: ModelKind[],
    progressRef: typeof transcriptionProgress,
  ) {
    const current = status.value;
    if (busy.value || !current) return;
    busy.value = kind;
    const parts = kinds.map((k) => ({ kind: k, size: current.models[k].downloadSize }));
    const total = parts.reduce((sum, part) => sum + part.size, 0);
    const downloadState: RuntimeDownloadState = {
      jobId: `runtime-model:${crypto.randomUUID()}`,
      cancelled: false,
    };
    activeDownload.value = downloadState;
    const cancelDownload = async () => {
      downloadState.cancelled = true;
      await cancelModelInstall(downloadState.jobId);
    };
    const taskId = tasks.start(title, undefined, cancelDownload, downloadState.jobId);
    downloadState.taskId = taskId;

    try {
      await downloadMultiPartModel({
        jobId: downloadState.jobId,
        parts,
        total,
        isCancelled: () => downloadState.cancelled,
        onProgress: (info) => {
          progressRef.value = info;
        },
        onTaskProgress: (pct, detail) => tasks.progress(taskId, pct, detail),
      });
      progressRef.value.state = "idle";
      tasks.succeed(taskId);
      toast.success(t("settings.offlineModels.download.complete", { model: title }));
    } catch (error) {
      if (downloadState.cancelled) {
        progressRef.value = initialModelProgress();
        return;
      }
      progressRef.value.state = "error";
      progressRef.value.error = String(error);
      tasks.fail(taskId, String(error));
      toast.error(String(error));
    } finally {
      try {
        status.value = await getLocalResourcesStatus();
      } catch (error) {
        toast.error(String(error));
      }
      busy.value = undefined;
      activeDownload.value = undefined;
    }
  }

  async function remove(kind: RuntimeActionKind, kinds: ModelKind[]) {
    if (busy.value) return;
    busy.value = kind;
    try {
      await Promise.all(kinds.map((k) => removeLocalModel(k)));
      status.value = await getLocalResourcesStatus();
    } catch (error) {
      toast.error(String(error));
    } finally {
      busy.value = undefined;
    }
  }

  onMounted(() => void refresh());

  return {
    status,
    busy,
    transcriptionInstalled,
    transcriptionInstalledCount: transcriptionStats.count,
    transcriptionTotalCount: transcriptionStats.totalCount,
    transcriptionSize: transcriptionStats.size,
    textEmbeddingInstalled,
    textEmbeddingInstalledCount: textEmbeddingStats.count,
    textEmbeddingTotalCount: textEmbeddingStats.totalCount,
    textEmbeddingSize: textEmbeddingStats.size,
    transcriptionProgress,
    textEmbeddingProgress,
    cancelDownload: async () => {
      const current = activeDownload.value;
      if (current?.taskId) await tasks.cancel(current.taskId);
    },
    refresh,
    downloadTranscription: () =>
      download(
        "transcription",
        t("settings.offlineModels.models.whisperStandard"),
        TRANSCRIPTION_KINDS,
        transcriptionProgress,
      ),
    downloadTextEmbedding: () =>
      download(
        "textEmbedding",
        t("settings.offlineModels.models.textEmbedding.label"),
        TEXT_EMBEDDING_KINDS,
        textEmbeddingProgress,
      ),
    removeTranscription: () => remove("transcription", TRANSCRIPTION_KINDS),
    removeTextEmbedding: () => remove("textEmbedding", TEXT_EMBEDDING_KINDS),
  };
}
