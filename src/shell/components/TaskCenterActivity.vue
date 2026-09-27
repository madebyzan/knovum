<script setup lang="ts">
import { CircleCheck, CircleStop, CircleX, RotateCcw, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { AppTask } from "../../foundation/tasks/task-store";
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
</script>

<template>
  <div
    class="max-h-[min(28rem,70vh)] overflow-x-hidden overflow-y-auto overscroll-contain"
    aria-live="polite"
  >
    <div
      v-for="task in tasks"
      :key="task.id"
      class="grid grid-cols-[1rem_minmax(0,1fr)_1.75rem] items-center gap-x-2 border-t border-divider px-3 py-2.5 first:border-t-0"
    >
      <UiSpinner v-if="task.status === 'running'" size="sm" tone="accent" />
      <CircleCheck
        v-else-if="task.status === 'succeeded'"
        class="size-(--knovum-icon-size-xs) text-success"
      />
      <CircleStop
        v-else-if="task.status === 'cancelled'"
        class="size-(--knovum-icon-size-xs) text-foreground-secondary"
      />
      <CircleX v-else class="size-(--knovum-icon-size-xs) text-danger" />
      <button
        v-if="task.openable"
        type="button"
        :title="task.title"
        class="min-w-0 truncate rounded-control text-left text-label hover:text-accent focus-visible:outline-2 focus-visible:outline-focus-ring"
        @click="emit('open', task.id)"
      >
        {{ task.title }}
      </button>
      <span v-else class="min-w-0 truncate text-label" :title="task.title">{{ task.title }}</span>
      <UiIconButton
        v-if="task.status === 'running' && task.cancellable"
        :label="t('tasks.cancel')"
        size="sm"
        @click="emit('cancel', task.id)"
      >
        <CircleStop class="size-(--knovum-icon-size-xs)" />
      </UiIconButton>
      <div v-else-if="task.status !== 'running'" class="flex items-center justify-end gap-0.5">
        <UiIconButton
          v-if="task.status === 'failed' && task.retryable"
          :label="t('common.retry')"
          size="sm"
          @click="emit('retry', task.id)"
        >
          <RotateCcw class="size-(--knovum-icon-size-xs)" />
        </UiIconButton>
        <UiIconButton :label="t('tasks.dismiss')" size="sm" @click="emit('dismiss', task.id)">
          <X class="size-(--knovum-icon-size-xs)" />
        </UiIconButton>
      </div>
      <UiIconButton
        v-else-if="task.cancelling"
        :label="t('tasks.forceDismiss')"
        size="sm"
        @click="emit('dismiss', task.id)"
      >
        <X class="size-(--knovum-icon-size-xs)" />
      </UiIconButton>
      <span v-else />
      <p
        v-if="task.cancelling"
        class="col-start-2 col-end-4 mt-1 truncate text-caption text-foreground-tertiary"
      >
        {{ t("tasks.cancelling") }}
      </p>
      <p
        v-else-if="task.statusMessage && task.status !== 'failed'"
        class="col-start-2 col-end-4 mt-1 text-caption text-foreground-tertiary"
        :class="task.status === 'running' ? 'truncate' : 'line-clamp-2 wrap-break-word'"
      >
        {{ task.statusMessage }}
      </p>
      <div v-else-if="task.status === 'failed'" class="col-start-2 col-end-4 mt-1 min-w-0">
        <p class="line-clamp-3 text-caption wrap-break-word text-foreground-secondary select-text">
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
