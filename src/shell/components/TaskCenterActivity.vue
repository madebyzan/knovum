<script setup lang="ts">
import {
  ArrowUpRight,
  CircleAlert,
  CircleCheck,
  CircleStop,
  CircleX,
  Clock3,
  RotateCcw,
  X,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { AppTask, TaskStatus } from "../../foundation/tasks/task-model";
import UiIconButton from "../../shared/ui/UiIconButton.vue";
import UiProgress from "../../shared/ui/UiProgress.vue";
import UiSpinner from "../../shared/ui/UiSpinner.vue";

defineProps<{ tasks: readonly AppTask[] }>();
const emit = defineEmits<{
  cancel: [taskId: string];
  dismiss: [taskId: string];
  retry: [taskId: string];
  open: [taskId: string];
}>();
const { t } = useI18n();

const active = (status: TaskStatus) => status === "queued" || status === "running";
const canCancel = (status: TaskStatus) => active(status) || status === "attention";
const terminal = (status: TaskStatus) => status === "succeeded" || status === "cancelled";

function statusClass(status: TaskStatus) {
  if (status === "failed") return "text-fg-danger";
  if (status === "succeeded") return "text-fg-success";
  if (status === "running") return "text-fg-accent";
  return "text-fg-tertiary";
}

function statusLabel(status: TaskStatus) {
  switch (status) {
    case "queued":
      return t("tasks.status.queued");
    case "running":
      return t("tasks.status.running");
    case "attention":
      return t("tasks.status.attention");
    case "succeeded":
      return t("tasks.status.succeeded");
    case "failed":
      return t("tasks.status.failed");
    case "cancelled":
      return t("tasks.status.cancelled");
  }
}
</script>

<template>
  <div
    class="max-h-[min(28rem,70vh)] overflow-x-hidden overflow-y-auto overscroll-contain"
    aria-live="polite"
  >
    <div
      v-for="task in tasks"
      :key="task.id"
      class="grid grid-cols-[1rem_minmax(0,1fr)_auto] items-start gap-x-2 border-t border-stroke-divider px-3 py-2.5 first:border-t-0"
    >
      <Clock3 v-if="task.status === 'queued'" class="mt-1 size-3 text-fg-tertiary" aria-hidden="true" />
      <UiSpinner v-else-if="task.status === 'running'" class="mt-1" size="sm" tone="accent" />
      <CircleAlert
        v-else-if="task.status === 'attention'"
        class="mt-1 size-3 text-fg-secondary"
        aria-hidden="true"
      />
      <CircleCheck
        v-else-if="task.status === 'succeeded'"
        class="mt-1 size-3 text-fg-success"
        aria-hidden="true"
      />
      <CircleStop
        v-else-if="task.status === 'cancelled'"
        class="mt-1 size-3 text-fg-secondary"
        aria-hidden="true"
      />
      <CircleX v-else class="mt-1 size-3 text-fg-danger" aria-hidden="true" />
      <div class="min-w-0">
        <div class="flex min-w-0 items-center gap-2">
          <button
            v-if="task.openable"
            type="button"
            :title="task.title"
            class="min-w-0 flex-1 truncate rounded-md text-left text-label hover:text-fg-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
            @click="emit('open', task.id)"
          >
            {{ task.title }}
          </button>
          <span v-else class="min-w-0 flex-1 truncate text-label" :title="task.title">
            {{ task.title }}
          </span>
          <span class="shrink-0 text-caption-sm" :class="statusClass(task.status)">
            {{ statusLabel(task.status) }}
          </span>
        </div>
        <p v-if="task.scope || task.subject" class="mt-0.5 truncate text-caption text-fg-tertiary">
          <template v-if="task.scope">
            <span class="sr-only">{{ t("tasks.scope") }}: </span>{{ task.scope.label }}
          </template>
          <span v-if="task.scope && task.subject" aria-hidden="true"> · </span>
          <template v-if="task.subject">
            <span class="sr-only">{{ t("tasks.subject") }}: </span>
            <span v-if="task.subject.typeLabel">{{ task.subject.typeLabel }} · </span>
            {{ task.subject.label }}
          </template>
        </p>
      </div>
      <div class="flex items-center justify-end gap-0.5">
        <UiIconButton
          v-if="task.status === 'attention' && task.openable"
          :label="t('tasks.openSubject')"
          size="sm"
          @click="emit('open', task.id)"
        >
          <ArrowUpRight class="size-3" />
        </UiIconButton>
        <UiIconButton
          v-if="canCancel(task.status) && task.cancellable"
          :label="t('tasks.cancel')"
          size="sm"
          @click="emit('cancel', task.id)"
        >
          <CircleStop class="size-3" />
        </UiIconButton>
        <UiIconButton
          v-if="(task.status === 'failed' || task.status === 'attention') && task.retryable"
          :label="t('common.retry')"
          size="sm"
          @click="emit('retry', task.id)"
        >
          <RotateCcw class="size-3" />
        </UiIconButton>
        <UiIconButton
          v-if="!active(task.status) && (!task.durable || terminal(task.status))"
          :label="t('tasks.dismiss')"
          size="sm"
          @click="emit('dismiss', task.id)"
        >
          <X class="size-3" />
        </UiIconButton>
        <UiIconButton
          v-if="task.cancelling && !task.durable"
          :label="t('tasks.forceDismiss')"
          tooltip
          size="sm"
          @click="emit('dismiss', task.id)"
        >
          <X class="size-3" />
        </UiIconButton>
      </div>
      <p v-if="task.cancelling" class="col-start-2 col-end-4 mt-1 truncate text-caption text-fg-tertiary">
        {{ t("tasks.cancelling") }}
      </p>
      <p
        v-else-if="task.statusMessage && task.status !== 'failed'"
        class="col-start-2 col-end-4 mt-1 text-caption text-fg-tertiary"
        :class="active(task.status) ? 'truncate' : 'line-clamp-2 wrap-break-word'"
      >
        {{ task.statusMessage }}
      </p>
      <div v-else-if="task.status === 'failed'" class="col-start-2 col-end-4 mt-1 min-w-0">
        <p class="line-clamp-3 text-caption wrap-break-word text-fg-secondary select-text">
          {{ task.statusMessage ?? t("tasks.failed") }}
        </p>
      </div>
      <div
        v-if="task.status === 'running' && task.progress !== undefined"
        class="col-start-2 col-end-4 mt-2 min-w-0"
      >
        <UiProgress :value="task.progress" :label="task.title" />
      </div>
    </div>
  </div>
</template>
