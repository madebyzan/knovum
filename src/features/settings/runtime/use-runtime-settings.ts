import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { sumBy } from "es-toolkit";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import {
  cancelModelInstall,
  getLocalResourcesStatus,
  removeLocalModel,
  textEmbeddingReady,
  transcriptionReady,
  type ModelKind,
} from "../../../foundation/runtime/local-resources";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { downloadMultiPartModel, initialModelProgress, type ModelProgressInfo } from "./model-download";

export type { ModelProgressInfo };

type RuntimeDownloadState = { jobId: string; taskId?: string; cancelled: boolean };

const LOCAL_RESOURCES_KEY = ["runtime", "local-resources"] as const;
const TRANSCRIPTION_KINDS: ModelKind[] = ["transcription", "transcriptionVad"];
const TEXT_EMBEDDING_KINDS: ModelKind[] = [
  "textEmbeddingConfig",
  "textEmbeddingTokenizer",
  "textEmbeddingWeights",
];

export function useRuntimeSettings() {
  const { t } = useI18n();
  const tasks = useTaskStore();
  const queryClient = useQueryClient();
  const invalidateStatus = () => queryClient.invalidateQueries({ queryKey: LOCAL_RESOURCES_KEY });
  const statusQuery = useQuery({ queryKey: LOCAL_RESOURCES_KEY, queryFn: getLocalResourcesStatus });
  const removal = useMutation({
    mutationFn: (kinds: ModelKind[]) => Promise.all(kinds.map((kind) => removeLocalModel(kind))),
    onSuccess: invalidateStatus,
    onError: (error) => toast.error(String(error)),
  });
  const activeDownload = ref<RuntimeDownloadState>();
  const status = computed(() => statusQuery.data.value);
  const busy = computed(
    () => statusQuery.isFetching.value || removal.isPending.value || activeDownload.value !== undefined,
  );

  const transcriptionProgress = ref<ModelProgressInfo>(initialModelProgress());
  const textEmbeddingProgress = ref<ModelProgressInfo>(initialModelProgress());

  const transcriptionInstalled = computed(() => transcriptionReady(status.value));
  const textEmbeddingInstalled = computed(() => textEmbeddingReady(status.value));

  const modelStats = (kinds: ModelKind[]) => ({
    count: computed(() => {
      const current = status.value;
      return current ? sumBy(kinds, (kind) => Number(current.models[kind].verified)) : 0;
    }),
    totalCount: kinds.length,
    size: computed(() => {
      const current = status.value;
      return current ? sumBy(kinds, (kind) => current.models[kind].downloadSize) : 0;
    }),
  });

  const transcriptionStats = modelStats(TRANSCRIPTION_KINDS);
  const textEmbeddingStats = modelStats(TEXT_EMBEDDING_KINDS);

  watch(statusQuery.error, (error) => {
    if (error) toast.error(String(error));
  });

  async function download(title: string, kinds: ModelKind[], progressRef: typeof transcriptionProgress) {
    const current = status.value;
    if (busy.value || !current) return;
    const parts = kinds.map((kind) => ({ kind, size: current.models[kind].downloadSize }));
    const total = sumBy(parts, ({ size }) => size);
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
      activeDownload.value = undefined;
      await invalidateStatus();
    }
  }

  function remove(kinds: ModelKind[]) {
    if (busy.value) return;
    removal.mutate(kinds);
  }

  return {
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
    downloadTranscription: () =>
      download(
        t("settings.offlineModels.models.whisperStandard"),
        TRANSCRIPTION_KINDS,
        transcriptionProgress,
      ),
    downloadTextEmbedding: () =>
      download(
        t("settings.offlineModels.models.textEmbedding.label"),
        TEXT_EMBEDDING_KINDS,
        textEmbeddingProgress,
      ),
    removeTranscription: () => remove(TRANSCRIPTION_KINDS),
    removeTextEmbedding: () => remove(TEXT_EMBEDDING_KINDS),
  };
}
