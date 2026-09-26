<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoTextStyle } from "../../video-editing/project";
import SpaceVideoFontPicker from "./SpaceVideoFontPicker.vue";

interface InstalledFont {
  family: string;
  weights: number[];
}

let catalogRequest: Promise<InstalledFont[]> | undefined;

function installedFonts(): Promise<InstalledFont[]> {
  catalogRequest ??= invoke<InstalledFont[]>("plugin:spaces|list_video_fonts").catch((error: unknown) => {
    catalogRequest = undefined;
    throw error;
  });
  return catalogRequest;
}

const props = defineProps<{ style: VideoTextStyle }>();
const emit = defineEmits<{ change: [style: Partial<VideoTextStyle>] }>();
const { t } = useI18n();
const fonts = shallowRef<InstalledFont[]>([]);
const error = shallowRef("");
const loading = shallowRef(true);
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

onMounted(async () => {
  try {
    fonts.value = await installedFonts();
    if (fonts.value.length === 0) error.value = t("spaces.media.fontCatalogEmpty");
  } catch (cause) {
    error.value = String(cause);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="grid grid-cols-2 gap-2">
    <SpaceVideoFontPicker
      v-model="family"
      :families="families"
      :label="$t('spaces.media.fontFamily')"
      :empty-label="$t('spaces.media.fontSearchEmpty')"
      :disabled="loading || !!error"
    />
    <div class="min-w-0">
      <span class="mb-1.5 block text-caption text-foreground-secondary">{{
        $t("spaces.media.fontWeight")
      }}</span>
      <UiSelect
        v-model="weight"
        :label="$t('spaces.media.fontWeight')"
        :options="weights"
        :disabled="loading || !!error || !selectedFont"
        size="sm"
      />
    </div>
    <p v-if="error" class="col-span-2 text-micro text-danger">{{ error }}</p>
    <p v-else-if="!loading && !selectedFont" class="col-span-2 text-micro text-danger">
      {{ $t("spaces.media.fontUnavailable", { family: style.family }) }}
    </p>
  </div>
</template>
