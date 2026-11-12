<script setup lang="ts">
import { Activity } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  OBJECT_STORAGE_ACCESS_KEY_ID,
  OBJECT_STORAGE_SECRET_ACCESS_KEY,
} from "../../../foundation/object-storage/object-storage-settings";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSettingsHeaderActions from "../../../shared/ui/UiSettingsHeaderActions.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import PreferenceField from "../components/PreferenceField.vue";
import SecretPreferenceField from "../components/SecretPreferenceField.vue";
import { useObjectStorageEditing } from "./use-object-storage-editing";

const { t } = useI18n();
const form = useObjectStorageEditing();

const fieldMessages = computed(() => {
  const messages: Record<string, string> = {};
  for (const [field, key] of Object.entries(form.fieldErrors.value)) messages[field] = t(key);
  return messages;
});

const connectionFields = computed(
  () =>
    [
      {
        key: "endpoint",
        labelKey: "settings.objectStorage.connection.endpoint",
        descriptionKey: "settings.objectStorage.connection.endpointDescription",
        placeholder: t("settings.objectStorage.connection.endpointPlaceholder"),
      },
      {
        key: "region",
        labelKey: "settings.objectStorage.connection.region",
        descriptionKey: "settings.objectStorage.connection.regionDescription",
        placeholder: t("settings.objectStorage.connection.regionPlaceholder"),
      },
      {
        key: "bucket",
        labelKey: "settings.objectStorage.connection.bucket",
        descriptionKey: "settings.objectStorage.connection.bucketDescription",
        placeholder: "knovum-media",
      },
      {
        key: "publicBaseUrl",
        labelKey: "settings.objectStorage.connection.publicBaseUrl",
        descriptionKey: "settings.objectStorage.connection.publicBaseUrlDescription",
        placeholder: "https://cdn.example.com",
      },
    ] as const,
);

async function test() {
  try {
    await form.test();
    toast.success(t("settings.objectStorage.actions.testSuccess"));
  } catch (error) {
    toast.error(`${t("settings.objectStorage.actions.testFailed")}: ${getErrorMessage(error)}`);
  }
}
</script>

<template>
  <div class="space-y-6" data-testid="object-storage-settings-pane">
    <UiSettingsHeaderActions>
      <UiButton
        size="sm"
        variant="secondary"
        :disabled="form.busy.value || !form.canTest.value"
        @click="test"
      >
        <UiSpinner v-if="form.testing.value" size="sm" />
        <Activity v-else class="size-3.5" />
        {{
          form.testing.value
            ? t("settings.objectStorage.status.testing")
            : t("settings.objectStorage.actions.test")
        }}
      </UiButton>
    </UiSettingsHeaderActions>

    <p class="max-w-2xl text-body text-fg-secondary">
      {{ t("settings.objectStorage.description") }}
    </p>

    <UiSettingsSection
      id="settings-object-storage-connection"
      :title="t('settings.objectStorage.connection.title')"
      :description="t('settings.objectStorage.connection.description')"
    >
      <PreferenceField
        v-for="field in connectionFields"
        :key="field.key"
        :label="t(field.labelKey)"
        :description="t(field.descriptionKey)"
      >
        <div class="flex flex-col items-end gap-1">
          <UiInput
            v-model="form.draft.value[field.key]"
            class="w-96"
            :class="fieldMessages[field.key] ? 'border-stroke-danger focus:border-stroke-danger' : ''"
            :disabled="form.busy.value"
            :placeholder="field.placeholder"
            :aria-label="t(field.labelKey)"
            @blur="form.saveField(field.key)"
            @commit="form.saveField(field.key)"
          />
          <span v-if="fieldMessages[field.key]" class="text-caption-sm text-fg-danger">
            {{ fieldMessages[field.key] }}
          </span>
        </div>
      </PreferenceField>
    </UiSettingsSection>

    <p v-if="form.loadError.value" class="text-caption text-fg-danger" role="alert">
      {{ t("settings.objectStorage.actions.loadFailed") }}: {{ form.loadError.value }}
    </p>

    <UiSettingsSection
      id="settings-object-storage-credentials"
      :title="t('settings.objectStorage.credentials.title')"
      :description="t('settings.objectStorage.credentials.description')"
    >
      <SecretPreferenceField
        :label="t('settings.objectStorage.credentials.accessKeyId')"
        :secret-key="OBJECT_STORAGE_ACCESS_KEY_ID"
        :disabled="form.busy.value"
        @change="form.onSecretChange('accessKeyId', $event)"
      />
      <SecretPreferenceField
        :label="t('settings.objectStorage.credentials.secretAccessKey')"
        :secret-key="OBJECT_STORAGE_SECRET_ACCESS_KEY"
        :disabled="form.busy.value"
        @change="form.onSecretChange('secretAccessKey', $event)"
      />
    </UiSettingsSection>
  </div>
</template>
