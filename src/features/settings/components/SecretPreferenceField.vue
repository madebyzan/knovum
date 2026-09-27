<script setup lang="ts">
import { Eye, EyeOff } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getSecret, persistOptionalSecret } from "../../../foundation/secrets/secret-store";
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
const dirty = shallowRef(false);
const saving = shallowRef(false);
const reveal = shallowRef(false);

function updateDraft(value: string) {
  draft.value = value;
  dirty.value = true;
}

async function persistDraft() {
  if (!dirty.value || saving.value) return;
  saving.value = true;
  try {
    await persistOptionalSecret(props.secretKey, draft.value);
    draft.value = "";
    dirty.value = false;
    reveal.value = false;
  } catch (error) {
    toast.error(`${t("settings.ai.integrationSaveFailed")}: ${String(error)}`);
  } finally {
    saving.value = false;
  }
}

async function toggleReveal() {
  try {
    if (!reveal.value && !draft.value) draft.value = (await getSecret(props.secretKey)) ?? "";
    reveal.value = !reveal.value;
  } catch (error) {
    toast.error(`${t("settings.ai.integrationSaveFailed")}: ${String(error)}`);
  }
}
</script>

<template>
  <PreferenceField :label="label" :description="description">
    <div class="relative w-80">
      <UiInput
        :model-value="draft"
        class="pr-10"
        :type="reveal ? 'text' : 'password'"
        placeholder="••••••••"
        :disabled="disabled || saving"
        @update:model-value="updateDraft"
        @blur="persistDraft"
        @keydown.enter="($event.target as HTMLInputElement).blur()"
      />
      <UiIconButton
        class="absolute top-1/2 right-1 -translate-y-1/2"
        :label="reveal ? t('common.hideSecret') : t('common.showSecret')"
        size="sm"
        @pointerdown.prevent
        @click="toggleReveal"
      >
        <EyeOff v-if="reveal" class="size-3.5" />
        <Eye v-else class="size-3.5" />
      </UiIconButton>
    </div>
  </PreferenceField>
</template>
