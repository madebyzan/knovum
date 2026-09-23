<script setup lang="ts">
import { useI18n } from "vue-i18n";

import SecretSettingsRow from "./SecretSettingsRow.vue";
import SettingsSection from "./SettingsSection.vue";

const FAL_IMAGE_LAYER_SECRET_KEY = "integrations.image-layers.fal.api-key";
const { t } = useI18n();
</script>

<template>
  <SettingsSection
    id="settings-image-layers"
    tabindex="-1"
    :title="t('settings.ai.imageLayersTitle')"
    :description="t('settings.ai.imageLayersDescription')"
  >
    <SecretSettingsRow
      :label="t('settings.ai.falApiKey')"
      :description="t('settings.ai.falApiKeyDescription')"
      :secret-key="FAL_IMAGE_LAYER_SECRET_KEY"
    />
  </SettingsSection>
</template>
