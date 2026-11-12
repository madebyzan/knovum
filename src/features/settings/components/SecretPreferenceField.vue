<script setup lang="ts">
import { Eye, EyeOff, RotateCcw } from "@lucide/vue";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { deleteSecret, isSecretConfigured, setSecret } from "../../../foundation/secrets/secret-store";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
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
const reveal = shallowRef(false);
const configured = shallowRef<boolean | null>(null);

onMounted(async () => {
  try {
    configured.value = await isSecretConfigured(props.secretKey);
    emit("change", Boolean(configured.value));
  } catch (error) {
    toast.error(`${t("settings.credentials.loadFailed")}: ${getErrorMessage(error)}`);
  }
});

async function save() {
  const trimmed = draft.value.trim();
  if (!trimmed || saving.value) return;
  saving.value = true;
  try {
    await setSecret(props.secretKey, trimmed);
    configured.value = true;
    draft.value = "";
    reveal.value = false;
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
    configured.value = false;
    draft.value = "";
    emit("change", false);
  } catch (error) {
    toast.error(`${t("settings.credentials.removeFailed")}: ${getErrorMessage(error)}`);
  } finally {
    saving.value = false;
  }
}

function handleBlur() {
  if (draft.value.trim()) void save();
}

function handleCommit() {
  if (draft.value.trim()) void save();
}
</script>

<template>
  <PreferenceField :label="label" :description="description">
    <div class="flex items-center gap-2">
      <div class="relative w-96">
        <UiInput
          v-model="draft"
          :aria-label="label"
          class="pr-16"
          :type="reveal ? 'text' : 'password'"
          :placeholder="
            configured === null
              ? ''
              : configured
                ? t('settings.credentials.savedPlaceholder')
                : t('settings.credentials.notSet')
          "
          :disabled="disabled || saving"
          @blur="handleBlur"
          @commit="handleCommit"
        />
        <div class="absolute top-1/2 right-1 flex -translate-y-1/2 items-center gap-0.5">
          <UiIconButton
            :label="reveal ? t('common.hideSecret') : t('common.showSecret')"
            size="sm"
            @click="reveal = !reveal"
          >
            <EyeOff v-if="reveal" class="size-3.5" />
            <Eye v-else class="size-3.5" />
          </UiIconButton>
          <UiIconButton
            v-if="configured"
            :label="t('settings.credentials.remove')"
            size="sm"
            :disabled="disabled || saving"
            @click="remove"
          >
            <RotateCcw class="size-3.5 text-fg-tertiary hover:text-fg-danger" />
          </UiIconButton>
        </div>
      </div>
      <span
        v-if="configured && !draft"
        class="shrink-0 rounded-sm bg-surface-success-subtle px-1.5 py-0.5 text-caption-sm text-fg-success"
      >
        {{ t("common.saved") }}
      </span>
    </div>
  </PreferenceField>
</template>
