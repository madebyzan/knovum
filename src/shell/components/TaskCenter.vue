<script setup lang="ts">
import { CircleCheck, CircleStop, CircleX } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useTaskStore } from "../../foundation/tasks/task-store";
import UiButton from "../../shared/ui/UiButton.vue";
import UiPopover from "../../shared/ui/UiPopover.vue";
import UiSpinner from "../../shared/ui/UiSpinner.vue";
import TaskCenterActivity from "./TaskCenterActivity.vue";

const store = useTaskStore();
const { completed, failed, tasks, running } = storeToRefs(store);
const { t } = useI18n();
const open = shallowRef(false);
const currentTask = computed(() => (running.value.length === 1 ? running.value[0] : undefined));

async function openTask(id: string) {
  try {
    await store.open(id);
    open.value = false;
  } catch (cause) {
    toast.error(String(cause));
  }
}

watch(
  () => tasks.value.length,
  (taskCount) => {
    if (!taskCount) open.value = false;
  },
);
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
    <TaskCenterActivity
      :tasks="tasks"
      @cancel="store.cancel"
      @dismiss="store.dismiss"
      @retry="store.retry"
      @open="openTask"
    />
  </UiPopover>
</template>
