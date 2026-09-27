<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { videoFontCatalogQuery } from "../../video-editing/font-catalog";
import type { VideoTextStyle } from "../../video-editing/project";
import SpaceVideoFontPicker from "./SpaceVideoFontPicker.vue";

const props = defineProps<{ style: VideoTextStyle }>();
const emit = defineEmits<{ change: [style: Partial<VideoTextStyle>] }>();
const { t } = useI18n();
const catalog = useQuery(videoFontCatalogQuery);
const fonts = computed(() => catalog.data.value ?? []);
const isPending = computed(() => catalog.isPending.value);
const error = computed(() => {
  if (catalog.error.value) return String(catalog.error.value);
  return isPending.value || fonts.value.length > 0 ? "" : t("spaces.media.videoEditor.font.catalogEmpty");
});
const selectedFont = computed(() => fonts.value.find(({ family }) => family === props.style.family));
const families = computed(() => fonts.value.map(({ family }) => family));
const weights = computed(() =>
  (selectedFont.value?.weights ?? []).map((weight) => ({
    value: String(weight),
    label: weightName(weight),
  })),
);
const family = computed({
  get: () => props.style.family,
  set: (value: string) => {
    const selected = fonts.value.find((font) => font.family === value);
    if (!selected) return;
    const weight = selected.weights.includes(props.style.weight)
      ? props.style.weight
      : selected.weights.includes(400)
        ? 400
        : selected.weights[0];
    if (weight !== undefined) emit("change", { family: value, weight });
  },
});
const weight = computed({
  get: () => String(props.style.weight),
  set: (value: string) => {
    const selected = Number(value);
    if (selectedFont.value?.weights.includes(selected)) emit("change", { weight: selected });
  },
});

function weightName(value: number): string {
  const names: Record<number, string> = {
    100: "Thin",
    200: "Extra Light",
    300: "Light",
    400: "Regular",
    500: "Medium",
    600: "Semibold",
    700: "Bold",
    800: "Extra Bold",
    900: "Black",
  };
  return names[value] ?? String(value);
}
</script>

<template>
  <div class="grid grid-cols-2 gap-2">
    <SpaceVideoFontPicker
      v-model="family"
      :families="families"
      :label="$t('spaces.media.videoEditor.font.family')"
      :empty-label="$t('spaces.media.videoEditor.font.searchEmpty')"
      :disabled="isPending || !!error"
    />
    <div class="min-w-0">
      <span class="mb-1.5 block text-caption text-fg-secondary">{{
        $t("spaces.media.videoEditor.font.weight")
      }}</span>
      <UiSelect
        v-model="weight"
        :label="$t('spaces.media.videoEditor.font.weight')"
        :options="weights"
        :disabled="isPending || !!error || !selectedFont"
        size="sm"
      />
    </div>
    <p v-if="error" class="col-span-2 text-caption-sm text-fg-danger">{{ error }}</p>
    <p v-else-if="!isPending && !selectedFont" class="col-span-2 text-caption-sm text-fg-danger">
      {{ $t("spaces.media.videoEditor.font.unavailable", { family: style.family }) }}
    </p>
  </div>
</template>
