<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { deleteSecret, isSecretConfigured, setSecret } from "../../../foundation/secrets/secret-store";
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

const placeholder = computed(() => {
  if (configured.value === true) return t("settings.credentials.configuredPlaceholder");
  if (configured.value === false) return t("settings.credentials.notSet");
  return "";
});

onMounted(async () => {
  try {
    const stored = await isSecretConfigured(props.secretKey);
    configured.value = stored;
    emit("change", stored);
  } catch (error) {
    toast.error(`${t("settings.credentials.loadFailed")}: ${getErrorMessage(error)}`);
  }
});

async function save() {
  if (saving.value) return;
  const trimmed = draft.value.trim();
  if (!trimmed) return;
  saving.value = true;
  try {
    await setSecret(props.secretKey, trimmed);
    draft.value = "";
    configured.value = true;
    emit("change", true);
  } catch (error) {
    toast.error(`${t("settings.credentials.saveFailed")}: ${getErrorMessage(error)}`);
  } finally {
    saving.value = false;
  }
}

async function remove() {
  if (saving.value) return;
  saving.value = true;
  try {
    await deleteSecret(props.secretKey);
    draft.value = "";
    configured.value = false;
    emit("change", false);
  } catch (error) {
    toast.error(`${t("settings.credentials.removeFailed")}: ${getErrorMessage(error)}`);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <PreferenceField :label="label" :description="description">
    <div class="flex items-center gap-2">
      <UiSecretInput
        v-model="draft"
        class="w-96"
        :aria-label="label"
        :placeholder="placeholder"
        :disabled="disabled || saving"
        @commit="save"
      />
      <button
        v-if="configured === true"
        type="button"
        :disabled="disabled || saving"
        class="rounded-sm text-caption text-fg-secondary underline-offset-2 hover:text-fg-danger hover:underline focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
        @click="remove"
      >
        {{ t("common.remove") }}
      </button>
    </div>
  </PreferenceField>
</template>
