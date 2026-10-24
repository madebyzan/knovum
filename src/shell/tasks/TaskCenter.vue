<script setup lang="ts">
import { CircleAlert, CircleCheck, CircleX, ListTodo } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../foundation/errors/error-format";
import type { TaskStatus } from "../../foundation/tasks/task-model";
import { useTaskStore } from "../../foundation/tasks/task-store";
import type { TaskReference } from "../../shared/contracts/task-reference";
import UiButton from "../../shared/ui/UiButton.vue";
import UiPopover from "../../shared/ui/UiPopover.vue";
import TaskActivityIndicator, { type TaskIndicatorVariant } from "./TaskActivityIndicator.vue";
import TaskCenterActivity from "./TaskCenterActivity.vue";

type TaskFilter = "all" | "current";

const props = withDefaults(
  defineProps<{
    currentScope?: TaskReference | undefined;
    variant?: TaskIndicatorVariant;
  }>(),
  {
    currentScope: undefined,
    variant: "waveform",
  },
);
const store = useTaskStore();
const { tasks } = storeToRefs(store);
const { t } = useI18n();
const open = shallowRef(false);
const filter = shallowRef<TaskFilter>("all");
// Work that has its own surface is tracked by the store but belongs there, not here: it is watched
// where it happens. Every count on this panel is a count of what the panel shows.
const listed = computed(() => tasks.value.filter((task) => task.listed !== false));
const byStatus = (status: TaskStatus) =>
  computed(() => listed.value.filter((task) => task.status === status));
const running = byStatus("running");
const queued = byStatus("queued");
const attention = byStatus("attention");
const failed = byStatus("failed");
const completed = byStatus("succeeded");
const activeCount = computed(() => running.value.length + queued.value.length);
const activeProgress = computed(() => {
  const taskWithProgress = running.value.find((task) => task.progress !== undefined);
  return taskWithProgress?.progress;
});
const visibleTasks = computed(() => {
  const scope = props.currentScope;
  if (filter.value === "all" || !scope) return listed.value;
  return listed.value.filter((task) => task.scope?.kind === scope.kind && task.scope?.id === scope.id);
});

async function openTask(taskId: string) {
  try {
    await store.open(taskId);
    open.value = false;
  } catch (cause) {
    toast.error(getErrorMessage(cause));
  }
}

watch(
  () => props.currentScope?.id,
  (scopeId) => {
    if (!scopeId) filter.value = "all";
  },
);
</script>

<template>
  <UiPopover
    v-model:open="open"
    align="end"
    side="bottom"
    :side-offset="7"
    class="w-80 overflow-hidden rounded-xl p-0"
  >
    <template #trigger>
      <button
        type="button"
        :aria-label="t('tasks.summary', { count: listed.length })"
        class="inline-flex h-(--knovum-control-height-md) min-w-(--knovum-control-height-md) items-center justify-center gap-1 rounded-md px-1.5 text-caption text-fg-secondary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus motion-reduce:transition-none"
      >
        <TaskActivityIndicator v-if="activeCount" :variant="variant" :progress="activeProgress" />
        <CircleX v-else-if="failed.length" class="size-3 text-fg-danger" />
        <CircleAlert v-else-if="attention.length" class="size-3 text-fg-secondary" />
        <CircleCheck v-else-if="completed.length" class="size-3 text-fg-success" />
        <ListTodo v-else class="size-3" />
        <span class="tabular-nums">{{ listed.length }}</span>
        <CircleX v-if="activeCount && failed.length" class="size-3 text-fg-danger" />
      </button>
    </template>

    <div class="border-b border-stroke-divider px-3 py-2">
      <div class="flex h-(--knovum-control-height-sm) items-center gap-2">
        <div class="min-w-0 flex-1 text-title-sm font-semibold tracking-title">
          {{ t("tasks.title") }}
        </div>
        <UiButton v-if="completed.length" size="sm" variant="ghost" @click="store.clearCompleted()">
          {{ t("tasks.clearCompleted") }}
        </UiButton>
      </div>
      <div
        class="mt-1 flex w-fit items-center rounded-md bg-surface-control p-0.5"
        role="group"
        :aria-label="t('tasks.scopeFilter')"
      >
        <button
          type="button"
          class="h-6 rounded-sm px-2 text-caption text-fg-secondary outline-none hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus aria-pressed:bg-surface-overlay aria-pressed:text-fg-primary aria-pressed:shadow-control"
          :aria-pressed="filter === 'all'"
          @click="filter = 'all'"
        >
          {{ t("tasks.all") }}
        </button>
        <button
          type="button"
          class="h-6 max-w-44 truncate rounded-sm px-2 text-caption text-fg-secondary outline-none hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus disabled:pointer-events-none disabled:opacity-40 aria-pressed:bg-surface-overlay aria-pressed:text-fg-primary aria-pressed:shadow-control"
          :aria-pressed="filter === 'current'"
          :disabled="!currentScope"
          @click="filter = 'current'"
        >
          {{ currentScope?.label ?? t("tasks.currentSpace") }}
        </button>
      </div>
    </div>
    <TaskCenterActivity
      :tasks="visibleTasks"
      @cancel="store.cancel"
      @dismiss="store.dismiss"
      @retry="store.retry"
      @open="openTask"
    />
    <p v-if="!visibleTasks.length" class="px-3 py-6 text-center text-caption text-fg-tertiary">
      {{ filter === "current" ? t("tasks.emptyCurrentSpace") : t("tasks.empty") }}
    </p>
  </UiPopover>
</template>
