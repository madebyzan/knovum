<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { ZodError } from "zod";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  OBJECT_STORAGE_ACCESS_KEY_ID,
  OBJECT_STORAGE_SECRET_ACCESS_KEY,
} from "../../../foundation/object-storage/object-storage-settings";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import PreferenceField from "../components/PreferenceField.vue";
import SecretPreferenceField from "../components/SecretPreferenceField.vue";
import { useObjectStorageForm } from "./use-object-storage-form";

const { t } = useI18n();
const form = useObjectStorageForm();

function describe(error: unknown) {
  if (error instanceof ZodError) return error.issues[0]?.message ?? getErrorMessage(error);
  return getErrorMessage(error);
}

async function save() {
  try {
    await form.save();
    toast.success(t("settings.objectStorage.actions.saved"));
  } catch (error) {
    toast.error(`${t("settings.objectStorage.actions.saveFailed")}: ${describe(error)}`);
  }
}

async function test() {
  try {
    await form.test();
    toast.success(t("settings.objectStorage.actions.testSuccess"));
  } catch (error) {
    toast.error(`${t("settings.objectStorage.actions.testFailed")}: ${describe(error)}`);
  }
}

async function remove() {
  const confirmed = await confirmAction({
    title: t("settings.objectStorage.actions.removeConfirm.title"),
    description: t("settings.objectStorage.actions.removeConfirm.description"),
    confirmLabel: t("settings.objectStorage.actions.removeConfirm.confirm"),
    danger: true,
  });
  if (!confirmed) return;
  try {
    await form.remove();
    toast.success(t("settings.objectStorage.actions.removed"));
  } catch (error) {
    toast.error(`${t("settings.objectStorage.actions.removeFailed")}: ${describe(error)}`);
  }
}
</script>

<template>
  <div class="space-y-6" data-testid="object-storage-settings-pane">
    <p class="max-w-2xl text-body text-fg-secondary">
      {{ t("settings.objectStorage.description") }}
    </p>
    <UiSettingsSection
      id="settings-object-storage-connection"
      :title="t('settings.objectStorage.connection.title')"
      :description="t('settings.objectStorage.connection.description')"
    >
      <PreferenceField :label="t('settings.objectStorage.connection.endpoint')">
        <UiInput
          v-model="form.draft.value.endpoint"
          class="w-80"
          :disabled="form.busy.value"
          :placeholder="t('settings.objectStorage.connection.endpointPlaceholder')"
          :aria-label="t('settings.objectStorage.connection.endpoint')"
        />
      </PreferenceField>
      <PreferenceField :label="t('settings.objectStorage.connection.region')">
        <UiInput
          v-model="form.draft.value.region"
          class="w-80"
          :disabled="form.busy.value"
          :placeholder="t('settings.objectStorage.connection.regionPlaceholder')"
          :aria-label="t('settings.objectStorage.connection.region')"
        />
      </PreferenceField>
      <PreferenceField :label="t('settings.objectStorage.connection.bucket')">
        <UiInput
          v-model="form.draft.value.bucket"
          class="w-80"
          :disabled="form.busy.value"
          placeholder="knovum-media"
          :aria-label="t('settings.objectStorage.connection.bucket')"
        />
      </PreferenceField>
      <PreferenceField
        :label="t('settings.objectStorage.connection.publicBaseUrl')"
        :description="t('settings.objectStorage.connection.publicBaseUrlDescription')"
      >
        <UiInput
          v-model="form.draft.value.publicBaseUrl"
          class="w-80"
          :disabled="form.busy.value"
          placeholder="https://cdn.example.com"
          :aria-label="t('settings.objectStorage.connection.publicBaseUrl')"
        />
      </PreferenceField>
      <div class="flex flex-wrap items-center gap-2 border-t border-stroke-divider px-4 py-3">
        <UiButton variant="primary" :disabled="form.busy.value" @click="save">
          {{ t("settings.objectStorage.actions.save") }}
        </UiButton>
        <UiButton :disabled="form.busy.value || !form.configured.value" @click="test">
          {{ t("settings.objectStorage.actions.test") }}
        </UiButton>
        <span
          v-if="form.configured.value"
          class="rounded-sm bg-surface-success-subtle px-1.5 py-0.5 text-caption-sm text-fg-success"
          >{{ t("settings.objectStorage.connection.configured") }}</span
        >
        <UiButton
          variant="dangerSubtle"
          class="ml-auto"
          :disabled="form.busy.value || !form.configured.value"
          @click="remove"
        >
          {{ t("settings.objectStorage.actions.remove") }}
        </UiButton>
      </div>
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
      />
      <SecretPreferenceField
        :label="t('settings.objectStorage.credentials.secretAccessKey')"
        :secret-key="OBJECT_STORAGE_SECRET_ACCESS_KEY"
        :disabled="form.busy.value"
      />
    </UiSettingsSection>
  </div>
</template>
