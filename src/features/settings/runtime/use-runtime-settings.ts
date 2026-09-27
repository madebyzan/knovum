import { useMutation } from "@tanstack/vue-query";
import { sumBy } from "es-toolkit";
import { computed, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import {
  removeLocalModel,
  textEmbeddingReady,
  transcriptionReady,
  type ModelKind,
  type ResourcePreparationProgress,
} from "../../../foundation/runtime/local-resources";
import { modelKinds, startModelPreparation } from "../../../foundation/runtime/model-preparation";
import {
  ResourcePreparationCancelledError,
  type ResourcePreparationHandle,
} from "../../../foundation/runtime/resource-preparation";
import { invalidateLocalResources, useLocalResourcesQuery } from "../../../foundation/runtime/resource-query";

export interface ModelProgressInfo {
  state: "idle" | "downloading" | "verifying" | "error";
  progress: number;
  downloaded: number;
  total: number;
  error?: string;
}

const TRANSCRIPTION_KINDS = modelKinds("transcription");
const TEXT_EMBEDDING_KINDS = modelKinds("textEmbedding");

export function useRuntimeSettings() {
  const { t } = useI18n();
  const statusQuery = useLocalResourcesQuery();
  const removal = useMutation({
    mutationFn: (kinds: ModelKind[]) => Promise.all(kinds.map((kind) => removeLocalModel(kind))),
    onSuccess: invalidateLocalResources,
    onError: (error) => toast.error(String(error)),
  });
  const activePreparation = shallowRef<ResourcePreparationHandle>();
  const status = computed(() => statusQuery.data.value);
  const busy = computed(
    () => statusQuery.isFetching.value || removal.isPending.value || activePreparation.value !== undefined,
  );

  const transcriptionProgress = ref<ModelProgressInfo>(initialProgress());
  const textEmbeddingProgress = ref<ModelProgressInfo>(initialProgress());
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

  async function download(
    capability: "transcription" | "textEmbedding",
    title: string,
    progressRef: typeof transcriptionProgress,
  ) {
    if (busy.value || !status.value) return;
    const handle = startModelPreparation(capability, title, (progress) => {
      if (progress.stage === "ready") return;
      progressRef.value = progressInfo(progress);
    });
    activePreparation.value = handle;
    try {
      await handle.completion;
      progressRef.value = initialProgress();
      toast.success(t("settings.offlineModels.download.complete", { model: title }));
    } catch (error) {
      if (error instanceof ResourcePreparationCancelledError) {
        progressRef.value = initialProgress();
        return;
      }
      progressRef.value = { ...progressRef.value, state: "error", error: String(error) };
      toast.error(String(error));
    } finally {
      activePreparation.value = undefined;
    }
  }

  function remove(kinds: ModelKind[]) {
    if (!busy.value) removal.mutate(kinds);
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
    cancelDownload: () => activePreparation.value?.cancel(),
    downloadTranscription: () =>
      download("transcription", t("settings.offlineModels.models.whisperStandard"), transcriptionProgress),
    downloadTextEmbedding: () =>
      download(
        "textEmbedding",
        t("settings.offlineModels.models.textEmbedding.label"),
        textEmbeddingProgress,
      ),
    removeTranscription: () => remove(TRANSCRIPTION_KINDS),
    removeTextEmbedding: () => remove(TEXT_EMBEDDING_KINDS),
  };
}

function initialProgress(): ModelProgressInfo {
  return { state: "idle", progress: 0, downloaded: 0, total: 0 };
}

function progressInfo(progress: ResourcePreparationProgress): ModelProgressInfo {
  return {
    state: progress.stage === "verifying" ? "verifying" : "downloading",
    progress: progress.percent ?? 0,
    downloaded: progress.downloaded,
    total: progress.total ?? 0,
  };
}
