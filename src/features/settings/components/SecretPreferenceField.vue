<script setup lang="ts">
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { deleteSecret, getSecret, setSecret } from "../../../foundation/secrets/secret-store";
import UiSecretInput from "../../../shared/ui/UiSecretInput.vue";
import PreferenceField from "./PreferenceField.vue";

const props = defineProps<{
  label: string;
  secretKey: string;
  description?: string;
  disabled?: boolean;
}>();
const emit = defineEmits<{ change: [configured: boolean] }>();

const { t } = useI18n();
const draft = shallowRef("");
const saving = shallowRef(false);
const configured = shallowRef<boolean | null>(null);

onMounted(async () => {
  try {
    const stored = await getSecret(props.secretKey);
    configured.value = stored !== null;
    draft.value = stored ?? "";
    emit("change", configured.value);
  } catch (error) {
    toast.error(`${t("settings.credentials.loadFailed")}: ${getErrorMessage(error)}`);
  }
});

async function save() {
  if (saving.value) return;
  const trimmed = draft.value.trim();
  saving.value = true;
  try {
    if (trimmed) await setSecret(props.secretKey, trimmed);
    else await deleteSecret(props.secretKey);
    draft.value = trimmed;
    configured.value = Boolean(trimmed);
    emit("change", configured.value);
  } catch (error) {
    const message = trimmed ? "settings.credentials.saveFailed" : "settings.credentials.removeFailed";
    toast.error(`${t(message)}: ${getErrorMessage(error)}`);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <PreferenceField :label="label" :description="description">
    <UiSecretInput
      v-model="draft"
      class="w-96"
      :aria-label="label"
      :placeholder="configured === false ? t('settings.credentials.notSet') : ''"
      :disabled="disabled || saving"
      @commit="save"
    />
  </PreferenceField>
</template>
