<script setup lang="ts">
import { Check, CircleAlert, LoaderCircle, Music2, RotateCcw, Square, X } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { useSourceSeparation } from "../../speech/use-source-separation";
import { workflowStateLabelKey } from "../../workflows/workflow-state-label";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";

const props = defineProps<{ separation: ReturnType<typeof useSourceSeparation> }>();
const emit = defineEmits<{ retry: []; close: [] }>();
const { t } = useI18n();
const running = computed(() => Boolean(props.separation.activeRunId.value));
const busy = computed(() => running.value || props.separation.isBusy.value);
const completed = computed(() => props.separation.runState.value === "succeeded");
const status = computed(() => {
  if (props.separation.isCancelling.value) return t("spaces.speech.separation.cancelling");
  if (props.separation.isPreparing.value) return t("spaces.speech.separation.starting");
  if (completed.value) return t("spaces.speech.separation.completed");
  const state = props.separation.runState.value;
  if (state === "running") return t("spaces.speech.separation.running");
  return state ? t(workflowStateLabelKey(state)) : t("spaces.speech.separation.title");
});
</script>

<template>
  <div class="max-w-full space-y-2" data-testid="node-separation-controls">
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      :aria-label="t('spaces.speech.separation.title')"
      :loop="false"
    >
      <component
        :is="busy ? LoaderCircle : completed ? Check : separation.error.value ? CircleAlert : Music2"
        class="ml-2 size-4.5 shrink-0"
        :class="[
          busy && 'animate-spin motion-reduce:animate-none',
          separation.error.value ? 'text-fg-danger' : 'text-fg-secondary',
        ]"
        :stroke-width="1.6"
      />
      <span
        class="min-w-0 truncate px-2 text-label text-fg-secondary"
        :role="separation.error.value ? 'alert' : 'status'"
        :title="status"
        >{{ status }}</span
      >
      <ToolbarButton v-if="running" as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :disabled="separation.isCancelling.value"
          data-testid="cancel-separation"
          @click="separation.cancel"
        >
          <Square class="size-4.5" :stroke-width="1.6" />{{ t("common.cancel") }}
        </button>
      </ToolbarButton>
      <ToolbarButton v-else-if="!busy && !completed" as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          data-testid="retry-separation"
          @click="emit('retry')"
        >
          <RotateCcw class="size-4.5" :stroke-width="1.6" />{{ t("common.retry") }}
        </button>
      </ToolbarButton>
      <ToolbarButton as-child>
        <UiIconButton size="md" :label="t('common.close')" tooltip @click="emit('close')"
          ><X class="size-4.5" :stroke-width="1.6"
        /></UiIconButton>
      </ToolbarButton>
    </ToolbarRoot>
    <p
      v-if="separation.error.value && separation.error.value !== status"
      role="alert"
      class="max-w-sm rounded-panel border border-stroke-divider bg-surface-overlay px-3 py-2 text-caption wrap-break-word text-fg-danger"
    >
      {{ separation.error.value }}
    </p>
  </div>
</template>
