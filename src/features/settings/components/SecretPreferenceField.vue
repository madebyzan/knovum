<script setup lang="ts">
import { Eye, EyeOff } from "@lucide/vue";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { deleteSecret, isSecretConfigured, setSecret } from "../../../foundation/secrets/secret-store";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import PreferenceField from "./PreferenceField.vue";

const props = defineProps<{
  label: string;
  secretKey: string;
  description?: string;
  disabled?: boolean;
}>();
const { t } = useI18n();
const draft = shallowRef("");
const saving = shallowRef(false);
const reveal = shallowRef(false);
const configured = shallowRef<boolean | null>(null);

onMounted(async () => {
  try {
    configured.value = await isSecretConfigured(props.secretKey);
  } catch (error) {
    toast.error(`${t("settings.ai.secretLoadFailed")}: ${getErrorMessage(error)}`);
  }
});

async function save() {
  if (!draft.value.trim() || saving.value) return;
  saving.value = true;
  try {
    await setSecret(props.secretKey, draft.value.trim());
    configured.value = true;
    draft.value = "";
    reveal.value = false;
  } catch (error) {
    toast.error(`${t("settings.ai.integrationSaveFailed")}: ${getErrorMessage(error)}`);
  } finally {
    saving.value = false;
  }
}

async function remove() {
  if (saving.value) return;
  saving.value = true;
  try {
    await deleteSecret(props.secretKey);
    configured.value = false;
    draft.value = "";
  } catch (error) {
    toast.error(`${t("settings.ai.integrationSaveFailed")}: ${getErrorMessage(error)}`);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <PreferenceField :label="label" :description="description">
    <div class="flex flex-wrap items-center gap-2">
      <div class="relative w-80">
        <UiInput
          v-model="draft"
          :aria-label="label"
          class="pr-10"
          :type="reveal ? 'text' : 'password'"
          :placeholder="
            configured === null
              ? ''
              : configured
                ? t('settings.ai.keySavedPlaceholder')
                : t('settings.ai.apiKeyNotSet')
          "
          :disabled="disabled || saving"
          @keydown.enter="save"
        />
        <UiIconButton
          class="absolute top-1/2 right-1 -translate-y-1/2"
          :label="reveal ? t('common.hideSecret') : t('common.showSecret')"
          size="sm"
          @click="reveal = !reveal"
        >
          <EyeOff v-if="reveal" class="size-3.5" />
          <Eye v-else class="size-3.5" />
        </UiIconButton>
      </div>
      <UiButton size="sm" :disabled="disabled || saving || !draft.trim()" @click="save">{{
        t("common.save")
      }}</UiButton>
      <UiButton v-if="configured" size="sm" variant="ghost" :disabled="disabled || saving" @click="remove">{{
        t("settings.ai.removeSavedKey")
      }}</UiButton>
    </div>
  </PreferenceField>
</template>
