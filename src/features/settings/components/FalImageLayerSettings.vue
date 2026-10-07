<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import SecretPreferenceField from "./SecretPreferenceField.vue";

const FAL_IMAGE_LAYER_SECRET_KEY = "integrations.image-layers.fal.api-key";
const { t } = useI18n();
</script>

<template>
  <UiSettingsSection
    id="settings-image-layers"
    tabindex="-1"
    :title="t('settings.ai.imageLayers.title')"
    :description="t('settings.ai.imageLayers.description')"
  >
    <SecretPreferenceField
      :label="t('settings.ai.imageLayers.apiKey')"
      :description="t('settings.ai.imageLayers.apiKeyDescription')"
      :secret-key="FAL_IMAGE_LAYER_SECRET_KEY"
    />
  </UiSettingsSection>
</template>
