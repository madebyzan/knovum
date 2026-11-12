<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { aiConnectionSecretKey } from "../../../../../foundation/ai/schema/models";
import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { getSecret } from "../../../../../foundation/secrets/secret-store";
import UiSecretInput from "../../../../../shared/ui/UiSecretInput.vue";
import type { AiConnectionSecretChange } from "../draft";

const secret = defineModel<AiConnectionSecretChange>({ required: true });
const props = defineProps<{ connectionId: string; existing: boolean; disabled: boolean }>();
const { t } = useI18n();
const hasKey = shallowRef<boolean | null>(props.existing ? null : false);
const loaded = shallowRef("");
const loadError = shallowRef("");

const display = computed(() =>
  secret.value === null ? "" : typeof secret.value === "string" ? secret.value : loaded.value,
);

onMounted(async () => {
  if (!props.existing) return;
  try {
    const stored = await getSecret(aiConnectionSecretKey(props.connectionId));
    loaded.value = stored ?? "";
    hasKey.value = stored !== null;
  } catch (error) {
    loadError.value = getErrorMessage(error);
  }
});

function handleInput(value: string) {
  if (value === loaded.value) secret.value = undefined;
  else if (value === "") secret.value = hasKey.value ? null : undefined;
  else secret.value = value;
}
</script>

<template>
  <div class="grid gap-1.5">
    <label :for="`key-${connectionId}`" class="text-caption font-medium text-fg-secondary">
      {{ t("settings.credentials.apiKey") }}
    </label>
    <UiSecretInput
      :id="`key-${connectionId}`"
      class="min-w-0"
      :model-value="display"
      :disabled="disabled"
      :aria-label="t('settings.credentials.apiKey')"
      :placeholder="secret === null || hasKey === false ? t('settings.credentials.optionalPlaceholder') : ''"
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
