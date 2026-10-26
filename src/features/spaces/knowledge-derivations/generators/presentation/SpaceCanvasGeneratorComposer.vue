<script setup lang="ts">
import { ArrowUp, MousePointerClick, Plus, Upload } from "@lucide/vue";
import { useTextareaAutosize } from "@vueuse/core";
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../../foundation/ai/AiExecutionTargetField.vue";
import { isComposing } from "../../../../../foundation/dom/composition";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../../shared/ui/UiSelect.vue";
import { usePickerImport } from "../../../assets/use-picker-import";
import { useNodeOperationGate } from "../../../canvas/use-node-operation-gate";
import { canGenerate, generatorFrame, uploadPosition, type CanvasGenerator } from "../canvas-generator";
import { GENERATOR_KINDS } from "../generator-kinds";
import { useCanvasGeneratorStore } from "../generator-store";
import SpaceCanvasGeneratorReferences from "./SpaceCanvasGeneratorReferences.vue";
import { useReferenceReadings } from "./use-reference-readings";

/**
 * What a generator is asked, written beneath its frame: what it makes from, the prompt, the
 * output's shape, and who makes it. It stays screen-sized under the frame at any zoom.
 */
const props = defineProps<{ generator: CanvasGenerator; refused: string | undefined }>();
const { t } = useI18n();
const store = useCanvasGeneratorStore();
const operations = useNodeOperationGate();
const importFromPicker = usePickerImport();
const { textarea } = useTextareaAutosize();
const { references, sources, preparing, unreadable } = useReferenceReadings(() => props.generator);

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
const duration = computed({
  get: () => String(props.generator.durationSeconds),
  set: (value: string) => {
    const seconds = kind.value.durations.find((candidate) => String(candidate) === value);
    if (seconds) store.update(props.generator.id, { durationSeconds: seconds });
  },
});
const aspectRatios = computed(() => kind.value.aspectRatios.map((value) => ({ value, label: value })));
const durations = computed(() =>
  kind.value.durations.map((value) => ({ value: String(value), label: `${value}s` })),
);
const referenceMenu = computed(() => [
  {
    id: "canvas",
    label: t("spaces.creation.generator.references.fromCanvas"),
    icon: MousePointerClick,
    onSelect: () => store.startPicking(),
  },
  {
    id: "upload",
    label: t("spaces.creation.generator.references.upload"),
    icon: Upload,
    onSelect: () => void upload(),
  },
]);
/** Why Generate cannot run yet, when the reason is not obvious from an empty field. */
const blocked = computed(() => {
  if (preparing.value) return t("spaces.creation.generator.references.blocked.preparing");
  if (unreadable.value) return t("spaces.creation.generator.references.blocked.unreadable");
  return undefined;
});
const ready = computed(
  () => canGenerate(props.generator) && sources.value !== undefined && operations.allows("derive"),
);

async function upload() {
  if (!operations.allows("create")) return;
  const { id } = props.generator;
  const nodes = await importFromPicker({ position: uploadPosition(props.generator), reveal: false });
  store.addReferences(
    id,
    nodes.map((node) => node.id),
  );
}

function submit(event?: KeyboardEvent) {
  if (event && isComposing(event)) return;
  event?.preventDefault();
  if (ready.value && sources.value) void store.generate(props.generator.id, sources.value);
}

onMounted(() => {
  if (store.takePromptFocus(props.generator.id)) textarea.value?.focus({ preventScroll: true });
});
</script>

<template>
  <div
    data-canvas-generator
    class="generator-composer pointer-events-auto absolute"
    :style="{ left: `${frame.x + frame.width / 2}px`, top: `${frame.y + frame.height}px` }"
  >
    <div
      class="w-[30rem] overflow-hidden rounded-panel border bg-surface-raised shadow-popover transition-colors"
      :class="
        store.picking ? 'border-stroke-accent' : 'border-stroke-default focus-within:border-stroke-focus'
      "
    >
      <div
        v-if="store.picking"
        class="flex h-9 items-center gap-2 border-b border-stroke-accent bg-surface-accent-subtle pr-1.5 pl-3 text-caption"
        role="status"
      >
        <MousePointerClick class="size-3.5 shrink-0 text-fg-accent" :stroke-width="1.8" />
        <span class="min-w-0 flex-1 truncate" :class="refused ? 'text-fg-warning' : 'text-fg-accent'">
          {{
            refused
              ? t("spaces.creation.generator.references.refused", { title: refused })
              : t("spaces.creation.generator.references.picking")
          }}
        </span>
        <UiButton size="sm" variant="ghost" @click="store.stopPicking()">
          {{ t("spaces.creation.generator.references.done") }}
        </UiButton>
      </div>
      <SpaceCanvasGeneratorReferences
        v-if="references.length"
        :generator-id="generator.id"
        :references="references"
      />
      <textarea
        ref="textarea"
        v-model="prompt"
        rows="2"
        class="block max-h-48 min-h-16 w-full resize-none overflow-y-auto border-0 bg-transparent px-3.5 pt-3 pb-1 text-body text-fg-primary outline-none placeholder:text-fg-tertiary"
        :aria-label="t(`spaces.creation.generator.placeholder.${generator.kind}`)"
        :placeholder="t(`spaces.creation.generator.placeholder.${generator.kind}`)"
        @focus="store.stopPicking()"
        @keydown.enter.exact="submit"
        @keydown.escape="store.deactivate()"
      />
      <div class="flex min-w-0 items-center gap-1 px-2 pb-2">
        <UiDropdownMenu :items="referenceMenu" side="top" align="start" :side-offset="6">
          <template #trigger>
            <UiIconButton :label="t('spaces.creation.generator.references.add')" tooltip size="sm">
              <Plus class="size-4" :stroke-width="1.8" />
            </UiIconButton>
          </template>
        </UiDropdownMenu>
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
          v-model="duration"
          :label="t('spaces.creation.generator.duration')"
          :options="durations"
          size="sm"
          width="compact"
          class="border-transparent bg-transparent shadow-none hover:bg-surface-control-hover"
        />
        <AiExecutionTargetField
          v-model="target"
          :purpose="kind.purpose"
          compact
          class="ml-auto max-w-56 min-w-0"
        />
        <UiIconButton
          :label="blocked ?? t('spaces.creation.generator.generate')"
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
