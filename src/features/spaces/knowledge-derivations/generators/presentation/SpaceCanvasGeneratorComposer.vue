<script setup lang="ts">
import { ArrowUp, Plus, X } from "@lucide/vue";
import { useTextareaAutosize } from "@vueuse/core";
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../../foundation/ai/AiExecutionTargetField.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../../shared/ui/UiSelect.vue";
import UiTooltip from "../../../../../shared/ui/UiTooltip.vue";
import { spaceNodePreviewSource } from "../../../assets/node-preview-source";
import { nodeIcon } from "../../../canvas/presentation/node-icon";
import { useCanvasSelectionProjection } from "../../../canvas/use-canvas-selection-projection";
import { useNodeOperationGate } from "../../../canvas/use-node-operation-gate";
import { useSpaceStore } from "../../../session/space-store";
import { creationSources } from "../../catalog/creation-source";
import { GENERATOR_KINDS } from "../generator-kinds";
import { generatorFrame, useCanvasGeneratorStore, type CanvasGenerator } from "../generator-store";

/**
 * What a generator is asked, written beneath its frame: references from the canvas, the prompt,
 * the output's shape, and who makes it. It stays screen-sized under the frame at any zoom.
 */
const props = defineProps<{ generator: CanvasGenerator }>();
const { t } = useI18n();
const store = useCanvasGeneratorStore();
const space = useSpaceStore();
const operations = useNodeOperationGate();
const { selectedNodes } = useCanvasSelectionProjection(() => space.nodes);
const { textarea } = useTextareaAutosize();

const kind = computed(() => GENERATOR_KINDS[props.generator.kind]);
const frame = computed(() => generatorFrame(props.generator));
const prompt = computed({
  get: () => props.generator.prompt,
  set: (value: string) => store.update(props.generator.id, { prompt: value }),
});
const target = computed({
  get: () => props.generator.target,
  set: (value) => store.update(props.generator.id, { target: value }),
});
const aspectRatio = computed({
  get: () => props.generator.aspectRatio,
  set: (value: string) => store.update(props.generator.id, { aspectRatio: value }),
});
const aspectRatios = computed(() => kind.value.aspectRatios.map((value) => ({ value, label: value })));
const durations = computed(() =>
  kind.value.durations.map((value) => ({ value: String(value), label: `${value}s` })),
);
const references = computed(() =>
  props.generator.references.map((reference) => {
    const node = space.nodes.find(({ id }) => id === reference.nodeId);
    return {
      ...reference,
      icon: nodeIcon(node?.data.kind ?? "file"),
      preview: node && spaceNodePreviewSource(node.data),
    };
  }),
);
/** What is selected on the canvas and could still be added as a reference. */
const addable = computed(() => {
  const referenced = new Set(props.generator.references.map(({ nodeId }) => nodeId));
  return creationSources(selectedNodes.value).filter(({ nodeId }) => !referenced.has(nodeId));
});
const ready = computed(
  () => Boolean(props.generator.prompt.trim() && props.generator.target) && operations.allows("derive"),
);

function selectDuration(value: string) {
  const seconds = kind.value.durations.find((candidate) => String(candidate) === value);
  if (seconds) store.update(props.generator.id, { durationSeconds: seconds });
}

function submit(event?: KeyboardEvent) {
  if (event?.isComposing || event?.keyCode === 229) return;
  event?.preventDefault();
  if (ready.value) void store.generate(props.generator.id);
}

onMounted(() => textarea.value?.focus());
</script>

<template>
  <div
    data-canvas-generator
    class="generator-composer pointer-events-auto absolute"
    :style="{ left: `${frame.x + frame.width / 2}px`, top: `${frame.y + frame.height}px` }"
    @pointerdown.stop
  >
    <div
      class="w-[30rem] rounded-panel border border-stroke-default bg-surface-raised shadow-popover transition-colors focus-within:border-stroke-focus"
    >
      <div class="flex flex-wrap items-center gap-1.5 px-2.5 pt-2.5">
        <span
          v-for="reference in references"
          :key="reference.nodeId"
          class="group flex h-7 max-w-40 items-center gap-1.5 rounded-md border border-stroke-default bg-surface-control pr-1 pl-1 text-caption text-fg-secondary"
        >
          <img
            v-if="reference.preview"
            :src="reference.preview"
            alt=""
            class="size-5 shrink-0 rounded-sm object-cover"
          />
          <component :is="reference.icon" v-else class="size-3.5 shrink-0" :stroke-width="1.6" />
          <span class="min-w-0 truncate">{{ reference.title }}</span>
          <button
            type="button"
            class="grid size-4 shrink-0 place-items-center rounded-sm text-fg-tertiary hover:bg-surface-control-hover hover:text-fg-primary"
            :aria-label="t('spaces.creation.generator.references.remove', { title: reference.title })"
            @click="store.removeReference(generator.id, reference.nodeId)"
          >
            <X class="size-3" :stroke-width="1.8" />
          </button>
        </span>
        <UiTooltip
          :text="t('spaces.creation.generator.references.hint')"
          :disabled="addable.length > 0"
          side="top"
        >
          <button
            type="button"
            class="flex h-7 items-center gap-1 rounded-md border border-dashed border-stroke-strong px-2 text-caption text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary disabled:pointer-events-none disabled:opacity-40"
            :disabled="addable.length === 0"
            @click="store.addReferences(generator.id, addable)"
          >
            <Plus class="size-3.5" :stroke-width="1.8" />
            {{
              addable.length
                ? t("spaces.creation.generator.references.addSelected", { count: addable.length })
                : t("spaces.creation.generator.references.add")
            }}
          </button>
        </UiTooltip>
      </div>
      <textarea
        ref="textarea"
        v-model="prompt"
        rows="2"
        class="block max-h-48 min-h-16 w-full resize-none overflow-y-auto border-0 bg-transparent px-3 pt-2.5 pb-1 text-body text-fg-primary outline-none placeholder:text-fg-tertiary"
        :aria-label="t(`spaces.creation.generator.placeholder.${generator.kind}`)"
        :placeholder="t(`spaces.creation.generator.placeholder.${generator.kind}`)"
        @keydown.enter.exact="submit"
        @keydown.meta.enter="submit"
        @keydown.ctrl.enter="submit"
        @keydown.escape="store.deactivate()"
      />
      <div class="flex min-w-0 items-center gap-1 px-2 pb-2">
        <UiSelect
          v-if="aspectRatios.length > 1"
          v-model="aspectRatio"
          :label="t('spaces.creation.generator.aspectRatio')"
          :options="aspectRatios"
          size="sm"
          width="compact"
          class="border-transparent bg-transparent shadow-none hover:bg-surface-control-hover"
        />
        <UiSelect
          v-if="durations.length"
          :model-value="String(generator.durationSeconds)"
          :label="t('spaces.creation.generator.duration')"
          :options="durations"
          size="sm"
          width="compact"
          class="border-transparent bg-transparent shadow-none hover:bg-surface-control-hover"
          @update:model-value="selectDuration"
        />
        <AiExecutionTargetField
          v-model="target"
          :purpose="kind.purpose"
          compact
          class="ml-auto max-w-56 min-w-0"
        />
        <UiIconButton
          :label="t('spaces.creation.generator.generate')"
          tooltip
          size="sm"
          tone="primary"
          :disabled="!ready"
          @click="submit()"
        >
          <ArrowUp class="size-4" :stroke-width="2.25" />
        </UiIconButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Screen-sized beneath the frame: centred on it, a fixed gap below, whatever the zoom. */
.generator-composer {
  transform-origin: 0 0;
  transform: scale(calc(1 / var(--canvas-zoom))) translate(-50%, 12px);
}
</style>
