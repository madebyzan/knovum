<script setup lang="ts">
import { Check, LoaderCircle, Mic2, X } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";
import { computed, nextTick, shallowRef, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { CANVAS_ACTION_BAR_CLASSES } from "../../canvas/presentation/canvas-action-classes";
import {
  NODE_ACTION_LABEL_CLASS,
  nodeActionLabelState,
} from "../../canvas/presentation/selection/node-action-label";
import { useNodeActionBarDensity } from "../../canvas/presentation/selection/use-node-action-bar-density";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceVoiceCloneAuditionControls from "./SpaceVoiceCloneAuditionControls.vue";
import SpaceVoiceCloneSourceControls from "./SpaceVoiceCloneSourceControls.vue";
import { useCanvasVoiceClone } from "./use-canvas-voice-clone";

const props = defineProps<{ node: SpaceCanvasNode; availableWidth?: number | undefined }>();
const emit = defineEmits<{ close: [] }>();
const { t, locale } = useI18n();
const { form, reference, save } = useCanvasVoiceClone(toRef(props, "node"));
const playbackError = shallowRef<string>();
const referenceActionLabel = computed(() =>
  t(reference.jobId ? "common.cancel" : "spaces.speech.reference.prepare"),
);
const bar = useTemplateRef<HTMLElement>("bar");
const openPopover = shallowRef<"range" | "options" | "audition" | "candidates">();
const sourcePopover = computed({
  get: () =>
    openPopover.value === "range" || openPopover.value === "options" ? openPopover.value : undefined,
  set: (value: "range" | "options" | undefined) => {
    openPopover.value = value;
  },
});
const auditionPopover = computed({
  get: () =>
    openPopover.value === "audition" || openPopover.value === "candidates" ? openPopover.value : undefined,
  set: (value: "audition" | "candidates" | undefined) => {
    openPopover.value = value;
  },
});
const identity = computed(() => `${locale.value}:${props.node.id}:${Boolean(form.reference)}`);
const { compact, ready } = useNodeActionBarDensity(
  bar,
  computed(() => props.availableWidth ?? Number.POSITIVE_INFINITY),
  identity,
);
let focused = false;
watch(
  [ready, () => form.reference],
  async ([isReady, prepared], [, previous]) => {
    if (!isReady || (focused && (prepared === previous || document.activeElement !== document.body))) return;
    await nextTick();
    const target = prepared ? '[data-testid="clone-audition-options"]' : "button:not(:disabled)";
    bar.value?.querySelector<HTMLButtonElement>(target)?.focus({ preventScroll: true });
    focused = true;
  },
  { flush: "post" },
);
</script>

<template>
  <div
    ref="bar"
    class="w-max max-w-full"
    :class="ready ? 'visible' : 'invisible'"
    :style="{ maxWidth: availableWidth === undefined ? undefined : `${availableWidth}px` }"
  >
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      :aria-label="t('spaces.speech.voice.clone.fromReference')"
      :loop="false"
      data-testid="node-voice-clone-bar"
    >
      <template v-if="!compact">
        <Mic2 class="ml-2 size-4.5 shrink-0 text-fg-secondary" :stroke-width="1.6" />
        <span class="px-1.5 text-label font-medium">{{ t("spaces.speech.voice.clone.action") }}</span>
        <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
      </template>
      <SpaceVoiceCloneSourceControls
        v-model:open-popover="sourcePopover"
        :form="form"
        :reference="reference"
        :compact="compact"
        @playback-error="playbackError = $event"
      />
      <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
      <ToolbarButton v-if="!form.reference" as-child>
        <UiButton
          :variant="reference.jobId ? 'ghost' : 'primary'"
          class="mx-1"
          :disabled="reference.jobId ? reference.isCancelling : !reference.canPrepare"
          :aria-label="referenceActionLabel"
          :title="compact ? referenceActionLabel : undefined"
          @click="reference.jobId ? reference.cancel() : reference.prepare()"
        >
          <component
            :is="reference.jobId ? LoaderCircle : Check"
            class="size-4.5"
            :class="reference.jobId && 'animate-spin motion-reduce:animate-none'"
            :stroke-width="1.6"
          />
          <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]">{{
            referenceActionLabel
          }}</span>
        </UiButton>
      </ToolbarButton>
      <SpaceVoiceCloneAuditionControls
        v-else
        v-model:open-popover="auditionPopover"
        :compact="compact"
        :form="form"
        @save="save"
      />
      <ToolbarButton as-child
        ><UiIconButton :label="t('common.close')" size="md" tooltip @click="emit('close')"
          ><X class="size-4.5" /></UiIconButton
      ></ToolbarButton>
    </ToolbarRoot>
    <div
      v-if="reference.error || playbackError || form.errors.length"
      class="mt-2 max-w-sm space-y-1 rounded-panel border border-stroke-divider bg-surface-overlay px-3 py-2 text-caption wrap-break-word text-fg-danger"
      role="alert"
    >
      <p v-if="reference.error || playbackError">{{ reference.error || playbackError }}</p>
      <p v-for="message in form.errors" :key="message">{{ message }}</p>
    </div>
  </div>
</template>
