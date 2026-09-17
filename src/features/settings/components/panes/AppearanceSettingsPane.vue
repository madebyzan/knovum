<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import {
  type DensityPreference,
  type LocalePreference,
  type TextSizePreference,
  type ThemePreference,
  usePreferencesStore,
} from "../../../../foundation/preferences/preferences-store";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import SettingsRow from "../SettingsRow.vue";
import SettingsSection from "../SettingsSection.vue";

const { t } = useI18n();
const preferences = usePreferencesStore();
const { theme, density, textSize, locale } = storeToRefs(preferences);

const themes = computed(() => [
  { value: "system", label: t("common.system") },
  { value: "light", label: t("common.light") },
  { value: "dark", label: t("common.dark") },
]);
const textSizes = computed(() => [
  { value: "small", label: t("common.small") },
  { value: "default", label: t("common.default") },
  { value: "large", label: t("common.large") },
]);
const densities = computed(() => [
  { value: "compact", label: t("common.compact") },
  { value: "comfortable", label: t("common.comfortable") },
]);
const languages = computed(() => [
  { value: "system", label: t("common.system") },
  { value: "zh-CN", label: t("common.simplifiedChinese") },
  { value: "en", label: t("common.english") },
]);
</script>

<template>
  <SettingsSection :title="t('settings.appearance.title')">
    <SettingsRow
      id="settings-appearance-theme"
      tabindex="-1"
      :label="t('settings.appearance.theme')"
      :description="t('settings.appearance.themeDescription')"
    >
      <UiSelect
        :model-value="theme"
        :label="t('settings.appearance.theme')"
        :options="themes"
        @update:model-value="preferences.setTheme($event as ThemePreference)"
      />
    </SettingsRow>
    <SettingsRow
      id="settings-appearance-text-size"
      tabindex="-1"
      :label="t('settings.appearance.textSize')"
      :description="t('settings.appearance.textSizeDescription')"
    >
      <UiSelect
        :model-value="textSize"
        :label="t('settings.appearance.textSize')"
        :options="textSizes"
        @update:model-value="preferences.setTextSize($event as TextSizePreference)"
      />
    </SettingsRow>
    <SettingsRow
      id="settings-appearance-density"
      tabindex="-1"
      :label="t('settings.appearance.density')"
      :description="t('settings.appearance.densityDescription')"
    >
      <UiSelect
        :model-value="density"
        :label="t('settings.appearance.density')"
        :options="densities"
        @update:model-value="preferences.setDensity($event as DensityPreference)"
      />
    </SettingsRow>
    <SettingsRow
      id="settings-appearance-language"
      tabindex="-1"
      :label="t('settings.appearance.language')"
      :description="t('settings.appearance.languageDescription')"
    >
      <UiSelect
        :model-value="locale"
        :label="t('settings.appearance.language')"
        :options="languages"
        @update:model-value="preferences.setLocale($event as LocalePreference)"
      />
    </SettingsRow>
  </SettingsSection>
</template>
