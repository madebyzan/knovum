<script setup lang="ts">
import { CircleCheck, CircleStop, CircleX } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, onBeforeUnmount, onMounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import {
  cancelModelInstall,
  listModelInstalls,
  type ModelKind,
} from "../../foundation/runtime/local-resources";
import { useTaskStore } from "../../foundation/tasks/task-store";
import UiButton from "../../shared/ui/UiButton.vue";
import UiPopover from "../../shared/ui/UiPopover.vue";
import UiSpinner from "../../shared/ui/UiSpinner.vue";
import TaskCenterList from "./TaskCenterList.vue";

const store = useTaskStore();
const { completed, failed, tasks, running } = storeToRefs(store);
const { t } = useI18n();
const open = shallowRef(false);
const currentTask = computed(() => (running.value.length === 1 ? running.value[0] : undefined));

const runtimeModelTitleKeys: Record<ModelKind, string> = {
  transcription: "settings.runtime.whisperStandardModel",
  transcriptionVad: "settings.runtime.whisperStandardModel",
  textEmbeddingConfig: "settings.runtime.textEmbeddingModel",
  textEmbeddingTokenizer: "settings.runtime.textEmbeddingModel",
  textEmbeddingWeights: "settings.runtime.textEmbeddingModel",
};

let snapshotTimer: ReturnType<typeof setInterval> | undefined;
const restoredRuntimeTasks = new Set<string>();

async function restoreRuntimeDownloads() {
  const snapshots = await listModelInstalls();
  const activeIds = new Set(snapshots.map(({ jobId }) => jobId));
  for (const snapshot of snapshots) {
    const restored = store.restoreRunning(
      {
        id: snapshot.jobId,
        title: t(runtimeModelTitleKeys[snapshot.models[0]!]),
        startedAt: snapshot.startedAt,
      },
      () => cancelModelInstall(snapshot.jobId),
    );
    if (restored) restoredRuntimeTasks.add(snapshot.jobId);
  }
  for (const taskId of restoredRuntimeTasks) {
    if (activeIds.has(taskId)) continue;
    store.dismiss(taskId);
    restoredRuntimeTasks.delete(taskId);
  }
}

function stopRuntimePolling() {
  if (!snapshotTimer) return;
  clearInterval(snapshotTimer);
  snapshotTimer = undefined;
}

function startRuntimePolling() {
  stopRuntimePolling();
  snapshotTimer = globalThis.setInterval(() => void restoreRuntimeDownloads(), 1_000);
}

async function openTask(id: string) {
  try {
    await store.open(id);
    open.value = false;
  } catch (cause) {
    toast.error(String(cause));
  }
}

onMounted(() => {
  void restoreRuntimeDownloads();
});

watch(open, (isOpen) => {
  if (!isOpen) return stopRuntimePolling();
  void restoreRuntimeDownloads();
  startRuntimePolling();
});

watch(
  () => tasks.value.length,
  (taskCount) => {
    if (!taskCount) open.value = false;
  },
);

onBeforeUnmount(stopRuntimePolling);
</script>

<template>
  <UiPopover
    v-if="tasks.length"
    v-model:open="open"
    align="end"
    side="top"
    :side-offset="7"
    class="w-(--knovum-popover-width) overflow-hidden rounded-dialog p-0"
  >
    <template #trigger>
      <button
        type="button"
        :aria-label="t('tasks.title')"
        class="inline-flex h-(--knovum-control-height-sm) items-center gap-1.5 rounded-control border border-border bg-overlay px-2 text-caption text-foreground-secondary shadow-control transition-colors duration-100 ease-standard outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring motion-reduce:transition-none"
      >
        <UiSpinner v-if="running.length" size="sm" tone="accent" />
        <CircleX v-else-if="failed.length" class="size-(--knovum-icon-size-xs) text-danger" />
        <CircleCheck v-else-if="completed.length" class="size-(--knovum-icon-size-xs) text-success" />
        <CircleStop v-else class="size-(--knovum-icon-size-xs) text-foreground-secondary" />
        <span class="max-w-48 truncate">{{ currentTask ? currentTask.title : t("tasks.title") }}</span>
        <span v-if="currentTask?.progress !== undefined" class="tabular-nums"
          >{{ Math.round(currentTask.progress) }}%</span
        >
        <span v-else class="tabular-nums">{{ running.length || failed.length || tasks.length }}</span>
        <CircleX v-if="running.length && failed.length" class="size-(--knovum-icon-size-xs) text-danger" />
      </button>
    </template>

    <div class="flex h-(--knovum-toolbar-height) items-center gap-2 border-b border-divider px-3">
      <div class="min-w-0 flex-1 text-body-lg font-semibold tracking-title">
        {{ t("tasks.title") }}
      </div>
      <UiButton v-if="completed.length" size="sm" variant="ghost" @click="store.clearCompleted()">
        {{ t("tasks.clearCompleted") }}
      </UiButton>
    </div>
    <TaskCenterList
      :tasks="tasks"
      @cancel="store.cancel"
      @dismiss="store.dismiss"
      @retry="store.retry"
      @open="openTask"
    />
  </UiPopover>
</template>
