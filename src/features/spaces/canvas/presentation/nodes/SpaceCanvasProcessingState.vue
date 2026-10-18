<script setup lang="ts">
import { CircleAlert, X } from "@lucide/vue";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { useTaskStore } from "../../../../../foundation/tasks/task-store";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import SpaceCanvasWorkEffect from "./SpaceCanvasWorkEffect.vue";
import { useNodeWorkTask } from "./use-node-work-task";

const props = defineProps<{ nodeId: string; label: string; phase: "active" | "failed" }>();
const { t } = useI18n();
const tasks = useTaskStore();
const work = useNodeWorkTask(toRef(props, "nodeId"));
/** What is happening, said by the work itself when it belongs to this node. */
const message = computed(() => work.value?.statusMessage ?? props.label);
const progress = computed(() => (props.phase === "active" ? work.value?.progress : undefined));
</script>

<template>
  <div
    class="work-state-enter relative flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-surface-panel"
    role="status"
    aria-live="polite"
    :aria-label="message"
  >
    <SpaceCanvasWorkEffect v-if="phase === 'active'" placement="field" />
    <span
      v-if="phase === 'active'"
      class="relative inline-flex max-w-[90%] items-center gap-1.5 rounded-full bg-surface-inverse/75 py-1 pr-1 pl-3 text-caption font-medium text-fg-inverse"
    >
      <SpaceCanvasWorkEffect placement="inline" />
      <span class="truncate">{{ message }}</span>
      <UiIconButton
        v-if="work?.cancellable"
        :label="t('common.cancel')"
        size="sm"
        class="size-4 shrink-0 rounded-full text-fg-inverse/70 hover:bg-surface-inverse hover:text-fg-inverse"
        :disabled="work.cancelling"
        @pointerdown.stop
        @click.stop="tasks.cancel(work.id)"
      >
        <X class="size-3" />
      </UiIconButton>
      <span v-else class="pr-2" />
    </span>
    <span
      v-else
      class="relative inline-flex max-w-[90%] items-center gap-1.5 rounded-full border border-stroke-danger/20 bg-surface-panel px-3 py-1 text-caption font-medium text-fg-danger"
    >
      <CircleAlert class="size-3.5 shrink-0" :stroke-width="1.6" />
      <span class="truncate">{{ label }}</span>
    </span>
    <div
      v-if="progress !== undefined"
      class="absolute inset-x-0 bottom-0 h-0.5 bg-surface-inverse/15"
      role="presentation"
    >
      <div
        class="h-full bg-surface-inverse/60 transition-[width] duration-200 ease-standard motion-reduce:transition-none"
        :style="{ width: `${progress}%` }"
      />
    </div>
  </div>
</template>

<style scoped>
.work-state-enter {
  animation: work-state-enter 160ms ease-out both;
}

@keyframes work-state-enter {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .work-state-enter {
    animation: none;
  }
}
</style>
