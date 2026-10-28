<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../../shared/ui/UiSelect.vue";
import type { CanvasGenerator } from "../canvas-generator";
import { GENERATOR_KINDS } from "../generator-kinds";
import { useCanvasGeneratorStore } from "../generator-store";

/** The output's shape: its aspect ratio and, for motion, its duration, where the kind offers a choice. */
const props = defineProps<{ generator: CanvasGenerator }>();
const { t } = useI18n();
const store = useCanvasGeneratorStore();

const kind = computed(() => GENERATOR_KINDS[props.generator.kind]);
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
</script>

<template>
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
</template>
