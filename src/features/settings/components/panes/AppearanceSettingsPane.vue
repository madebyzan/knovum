<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { usePreferencesStore } from "../../../../foundation/preferences/preferences-store";
import type {
  DensityPreference,
  LocalePreference,
  TextSizePreference,
  ThemePreference,
} from "../../../../foundation/preferences/schema/preferences";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import PreferenceField from "../PreferenceField.vue";
import SettingsSection from "../SettingsSection.vue";

const { t } = useI18n();
const preferences = usePreferencesStore();
const { theme, density, textSize, locale } = storeToRefs(preferences);

const themes = computed(() => [
  { value: "system", label: t("settings.appearance.options.followSystem") },
  { value: "light", label: t("settings.appearance.options.theme.light") },
  { value: "dark", label: t("settings.appearance.options.theme.dark") },
]);
const textSizes = computed(() => [
  { value: "small", label: t("settings.appearance.options.textSize.small") },
  { value: "default", label: t("settings.appearance.options.textSize.default") },
  { value: "large", label: t("settings.appearance.options.textSize.large") },
]);
const densities = computed(() => [
  { value: "compact", label: t("settings.appearance.options.density.compact") },
  { value: "comfortable", label: t("settings.appearance.options.density.comfortable") },
]);
const languages = computed(() => [
  { value: "system", label: t("settings.appearance.options.followSystem") },
  { value: "zh-CN", label: t("settings.appearance.options.language.simplifiedChinese") },
  { value: "en", label: t("settings.appearance.options.language.english") },
]);
</script>

<template>
  <SettingsSection :title="t('settings.appearance.title')">
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
        @update:model-value="preferences.setTheme($event as ThemePreference)"
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
        @update:model-value="preferences.setTextSize($event as TextSizePreference)"
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
        @update:model-value="preferences.setDensity($event as DensityPreference)"
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
        @update:model-value="preferences.setLocale($event as LocalePreference)"
      />
    </PreferenceField>
  </SettingsSection>
</template>
