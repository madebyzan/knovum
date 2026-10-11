<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { usePreferencesStore } from "../../../../foundation/preferences/preferences-store";
import type {
  DensityPreference,
  LocalePreference,
  TextSizePreference,
  ThemePreference,
} from "../../../../foundation/preferences/schema/preferences";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SelectOption } from "../../../../shared/ui/UiSelect.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import PreferenceField from "../PreferenceField.vue";

const { t } = useI18n();
const preferences = usePreferencesStore();
const { theme, density, textSize, locale } = storeToRefs(preferences);

async function updatePreference(operation: () => Promise<void>) {
  try {
    await operation();
  } catch (error) {
    toast.error(`${t("settings.appearance.saveFailed")}: ${getErrorMessage(error)}`);
  }
}

const themes = computed<readonly SelectOption<ThemePreference>[]>(() => [
  { value: "system", label: t("settings.appearance.options.followSystem") },
  { value: "light", label: t("settings.appearance.options.theme.light") },
  { value: "dark", label: t("settings.appearance.options.theme.dark") },
]);
const textSizes = computed<readonly SelectOption<TextSizePreference>[]>(() => [
  { value: "small", label: t("settings.appearance.options.textSize.small") },
  { value: "default", label: t("settings.appearance.options.textSize.default") },
  { value: "large", label: t("settings.appearance.options.textSize.large") },
]);
const densities = computed<readonly SelectOption<DensityPreference>[]>(() => [
  { value: "compact", label: t("settings.appearance.options.density.compact") },
  { value: "comfortable", label: t("settings.appearance.options.density.comfortable") },
]);
const languages = computed<readonly SelectOption<LocalePreference>[]>(() => [
  { value: "system", label: t("settings.appearance.options.followSystem") },
  { value: "zh-CN", label: t("settings.appearance.options.language.simplifiedChinese") },
  { value: "zh-TW", label: t("ai.languages.zhTw") },
  { value: "en", label: t("settings.appearance.options.language.english") },
  { value: "ja", label: t("ai.languages.ja") },
  { value: "ko", label: t("ai.languages.ko") },
  { value: "fr", label: t("ai.languages.fr") },
  { value: "de", label: t("ai.languages.de") },
  { value: "es", label: t("ai.languages.es") },
  { value: "pt-BR", label: t("ai.languages.pt") },
  { value: "it", label: t("ai.languages.it") },
  { value: "ru", label: t("ai.languages.ru") },
]);
</script>

<template>
  <UiSettingsSection>
    <PreferenceField
      id="settings-appearance-theme"
      tabindex="-1"
      :label="t('settings.appearance.theme.label')"
      :description="t('settings.appearance.theme.description')"
    >
      <UiSelect
        :model-value="theme"
        :label="t('settings.appearance.theme.label')"
        :options="themes"
        @update:model-value="updatePreference(() => preferences.setTheme($event))"
      />
    </PreferenceField>
    <PreferenceField
      id="settings-appearance-text-size"
      tabindex="-1"
      :label="t('settings.appearance.textSize.label')"
      :description="t('settings.appearance.textSize.description')"
    >
      <UiSelect
        :model-value="textSize"
        :label="t('settings.appearance.textSize.label')"
        :options="textSizes"
        @update:model-value="updatePreference(() => preferences.setTextSize($event))"
      />
    </PreferenceField>
    <PreferenceField
      id="settings-appearance-density"
      tabindex="-1"
      :label="t('settings.appearance.density.label')"
      :description="t('settings.appearance.density.description')"
    >
      <UiSelect
        :model-value="density"
        :label="t('settings.appearance.density.label')"
        :options="densities"
        @update:model-value="updatePreference(() => preferences.setDensity($event))"
      />
    </PreferenceField>
    <PreferenceField
      id="settings-appearance-language"
      tabindex="-1"
      :label="t('settings.appearance.language.label')"
      :description="t('settings.appearance.language.description')"
    >
      <UiSelect
        :model-value="locale"
        :label="t('settings.appearance.language.label')"
        :options="languages"
        @update:model-value="updatePreference(() => preferences.setLocale($event))"
      />
    </PreferenceField>
  </UiSettingsSection>
</template>
