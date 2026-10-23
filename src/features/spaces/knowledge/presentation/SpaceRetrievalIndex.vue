<script setup lang="ts">
import { AlertTriangle, CheckCircle2, Database, LoaderCircle, RefreshCw } from "@lucide/vue";
import { useIntervalFn, whenever } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiProgress from "../../../../shared/ui/UiProgress.vue";
import { useSpaceStore } from "../../session/space-store";
import { readKnowledgeIndex } from "../indexing/index-reading";
import { isKnowledgeIndexingCancelled } from "../indexing/indexing-outcome";
import { knowledgeIndexingTaskId, startKnowledgeIndexing } from "../indexing/start-indexing";
import { useKnowledgeIndexStateQuery } from "../queries/use-knowledge-index-state-query";

const POLL_WHILE_INDEXING_MS = 2000;
const { t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const tasks = useTaskStore();
const { data, isFetching, error, refetch } = useKnowledgeIndexStateQuery(activeSpaceId);
const taskId = computed(() =>
  activeSpaceId.value ? knowledgeIndexingTaskId(activeSpaceId.value) : undefined,
);
// The pass this window started is live in the Task Center; a pass started by work that needed the
// index is only visible in the reading, which is why a Space being indexed keeps the panel polling.
const task = computed(() => tasks.tasks.find(({ id }) => id === taskId.value));
const ourPass = computed(() => task.value?.status === "running");
const reading = computed(() => readKnowledgeIndex(data.value, ourPass.value));
const progress = computed(() => (ourPass.value ? (task.value?.progress ?? 0) : reading.value.coverage));
const statusLabel = computed(() => {
  switch (reading.value.condition) {
    case "indexing":
      return t("spaces.knowledge.index.indexing");
    case "current":
      return t("spaces.knowledge.index.current");
    case "unbuilt":
      return t("spaces.knowledge.index.unbuilt");
    default:
      return t("spaces.knowledge.index.incomplete", { count: reading.value.pending });
  }
});
const actionLabel = computed(() =>
  reading.value.action === "build" ? t("spaces.knowledge.index.build") : t("spaces.knowledge.index.fill"),
);
const coverageLabel = computed(() =>
  t("spaces.knowledge.index.coverage", {
    indexed: reading.value.indexed,
    total: reading.value.total,
  }),
);
const poll = useIntervalFn(() => void refetch(), POLL_WHILE_INDEXING_MS, { immediate: false });

whenever(error, (cause) => toast.error(getErrorMessage(cause)));
watch(
  () => reading.value.condition === "indexing",
  (indexing) => (indexing ? poll.resume() : poll.pause()),
  { immediate: true },
);

async function index() {
  const spaceId = activeSpaceId.value;
  if (!spaceId || !reading.value.action) return;
  try {
    const { indexed } = await startKnowledgeIndexing(spaceId, t("spaces.knowledge.index.task")).completion;
    toast.success(
      indexed > 0
        ? t("spaces.knowledge.index.done", { count: indexed })
        : t("spaces.knowledge.index.upToDate"),
    );
  } catch (cause) {
    if (!isKnowledgeIndexingCancelled(cause)) toast.error(getErrorMessage(cause));
  } finally {
    void refetch();
  }
}
</script>

<template>
  <section>
    <div class="flex items-center justify-between gap-2">
      <h3 class="flex items-center gap-1.5 text-caption font-semibold text-fg-secondary">
        <Database class="size-3" />{{ t("spaces.knowledge.index.title") }}
      </h3>
      <UiIconButton
        :label="t('common.refresh')"
        size="sm"
        :disabled="isFetching"
        @click="() => void refetch()"
      >
        <RefreshCw class="size-3" :class="isFetching && 'animate-spin'" />
      </UiIconButton>
    </div>

    <p v-if="reading.condition === 'empty'" class="mt-1 text-caption text-fg-tertiary">
      {{ t("spaces.knowledge.index.empty") }}
    </p>
    <template v-else>
      <div class="mt-2 flex items-center gap-2 text-caption">
        <LoaderCircle
          v-if="reading.condition === 'indexing'"
          class="size-3.5 shrink-0 animate-spin text-fg-accent"
        />
        <CheckCircle2 v-else-if="reading.condition === 'current'" class="size-3.5 shrink-0 text-fg-success" />
        <AlertTriangle
          v-else
          class="size-3.5 shrink-0"
          :class="reading.condition === 'unbuilt' ? 'text-fg-danger' : 'text-fg-warning'"
        />
        <span class="min-w-0 flex-1 text-fg-primary">{{ statusLabel }}</span>
        <span class="shrink-0 text-caption-sm text-fg-tertiary tabular-nums">{{ progress }}%</span>
      </div>
      <UiProgress class="mt-2" :value="progress" :label="coverageLabel" />
      <p class="mt-1.5 text-caption-sm text-fg-tertiary">
        {{ coverageLabel
        }}<template v-if="reading.indexedAt">
          · {{ t("spaces.knowledge.index.indexedAt", { time: $d(reading.indexedAt, "medium") }) }}</template
        >
      </p>

      <UiButton
        v-if="reading.action"
        class="mt-3 w-full"
        size="sm"
        variant="secondary"
        @click="() => void index()"
      >
        {{ actionLabel }}
      </UiButton>
      <UiButton
        v-else-if="ourPass && taskId"
        class="mt-3 w-full"
        size="sm"
        variant="ghost"
        :disabled="task?.cancelling"
        @click="() => void tasks.cancel(taskId!)"
      >
        {{ t("common.cancel") }}
      </UiButton>
    </template>
  </section>
</template>
