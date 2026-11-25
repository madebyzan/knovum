<script setup lang="ts">
import { Languages } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";

defineProps<{ disabled?: boolean }>();
const language = defineModel<string>({ required: true });
const { t } = useI18n();

const LANGUAGE_ENTRIES = [
  ["zh", "ai.languages.zhCn"],
  ["en", "ai.languages.en"],
  ["ja", "ai.languages.ja"],
  ["ko", "ai.languages.ko"],
  ["fr", "ai.languages.fr"],
  ["de", "ai.languages.de"],
  ["es", "ai.languages.es"],
  ["pt", "ai.languages.pt"],
  ["it", "ai.languages.it"],
  ["ru", "ai.languages.ru"],
  ["ar", "ai.languages.ar"],
] as const;

const options = computed(() => [
  { value: "auto", label: t("spaces.canvas.sections.layout.auto") },
  ...LANGUAGE_ENTRIES.map(([value, labelKey]) => ({ value, label: t(labelKey) })),
]);
</script>

<template>
  <div class="mb-2 flex min-h-interaction items-center gap-2">
    <Languages class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
    <span class="min-w-0 flex-1 truncate text-caption text-fg-secondary">
      {{ t("settings.appearance.language.label") }}
    </span>
    <UiSelect
      v-model="language"
      size="sm"
      class="w-auto min-w-28"
      :disabled="disabled"
      :label="t('settings.appearance.language.label')"
      :options="options"
    />
  </div>
</template>
