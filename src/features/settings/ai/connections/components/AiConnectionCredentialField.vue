<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { aiConnectionSecretKey } from "../../../../../foundation/ai/schema/models";
import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { isSecretConfigured } from "../../../../../foundation/secrets/secret-store";
import UiSecretInput from "../../../../../shared/ui/UiSecretInput.vue";
import type { AiConnectionSecretChange } from "../draft";

const secret = defineModel<AiConnectionSecretChange>({ required: true });
const props = defineProps<{ connectionId: string; existing: boolean; disabled: boolean }>();
const { t } = useI18n();
const hasKey = shallowRef<boolean | null>(props.existing ? null : false);
const loadError = shallowRef("");

const display = computed(() => (typeof secret.value === "string" ? secret.value : ""));

const placeholder = computed(() => {
  if (secret.value === null || hasKey.value === false) return t("settings.credentials.optionalPlaceholder");
  if (hasKey.value === true && secret.value === undefined) {
    return t("settings.credentials.configuredPlaceholder");
  }
  return "";
});

onMounted(async () => {
  if (!props.existing) return;
  try {
    hasKey.value = await isSecretConfigured(aiConnectionSecretKey(props.connectionId));
  } catch (error) {
    loadError.value = getErrorMessage(error);
  }
});

function handleInput(value: string) {
  secret.value = value === "" ? undefined : value;
}
</script>

<template>
  <div class="grid gap-1.5">
    <div class="flex items-center justify-between gap-2">
      <label :for="`key-${connectionId}`" class="text-caption font-medium text-fg-secondary">
        {{ t("settings.credentials.apiKey") }}
      </label>
      <button
        v-if="hasKey === true && secret === undefined"
        type="button"
        :disabled="disabled"
        class="rounded-sm text-caption text-fg-secondary underline-offset-2 hover:text-fg-danger hover:underline focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
        @click="secret = null"
      >
        {{ t("common.remove") }}
      </button>
    </div>
    <UiSecretInput
      :id="`key-${connectionId}`"
      class="min-w-0"
      :model-value="display"
      :disabled="disabled"
      :aria-label="t('settings.credentials.apiKey')"
      :placeholder="placeholder"
      @update:model-value="handleInput"
    />
    <p v-if="secret === null" class="text-caption text-fg-secondary">
      {{ t("settings.credentials.removalPending") }}
      <button
        type="button"
        :disabled="disabled"
        class="rounded-sm underline-offset-2 hover:text-fg-primary hover:underline focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
        @click="secret = undefined"
      >
        {{ t("common.undo") }}
      </button>
    </p>
    <p v-if="loadError" class="text-caption text-fg-danger" role="alert">
      {{ t("settings.credentials.loadFailed") }}: {{ loadError }}
    </p>
  </div>
</template>
