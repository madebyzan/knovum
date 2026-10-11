<script setup lang="ts">
import { Eye, EyeOff } from "@lucide/vue";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { aiConnectionSecretKey } from "../../../../../foundation/ai/schema/models";
import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { isSecretConfigured } from "../../../../../foundation/secrets/secret-store";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../../shared/ui/UiInput.vue";
import type { AiConnectionSecretChange } from "../draft";

const secret = defineModel<AiConnectionSecretChange>({ required: true });
const props = defineProps<{ connectionId: string; existing: boolean; disabled: boolean }>();
const { t } = useI18n();
const hasKey = shallowRef<boolean | null>(props.existing ? null : false);
const reveal = shallowRef(false);
const loadError = shallowRef("");

onMounted(async () => {
  if (!props.existing) return;
  try {
    hasKey.value = await isSecretConfigured(aiConnectionSecretKey(props.connectionId));
  } catch (error) {
    loadError.value = getErrorMessage(error);
  }
});
</script>

<template>
  <div class="grid gap-1.5">
    <div class="flex items-center gap-2 text-caption font-medium text-fg-secondary">
      <label :for="`key-${connectionId}`">{{ t("settings.credentials.apiKey") }}</label>
      <span
        v-if="hasKey"
        class="rounded-sm bg-surface-success-subtle px-1.5 py-0.5 text-caption-sm text-fg-success"
        >{{ t("common.saved") }}</span
      >
    </div>
    <div class="relative min-w-0">
      <UiInput
        :id="`key-${connectionId}`"
        :model-value="typeof secret === 'string' ? secret : ''"
        :type="reveal ? 'text' : 'password'"
        :disabled="disabled"
        class="pr-10"
        :placeholder="
          hasKey === null
            ? ''
            : hasKey
              ? t('settings.credentials.savedPlaceholder')
              : t('settings.credentials.optionalPlaceholder')
        "
        @update:model-value="secret = $event || undefined"
      />
      <UiIconButton
        :label="reveal ? t('common.hideSecret') : t('common.showSecret')"
        :disabled="disabled"
        size="sm"
        class="absolute top-1/2 right-1 -translate-y-1/2"
        @click="reveal = !reveal"
      >
        <EyeOff v-if="reveal" class="size-3.5" />
        <Eye v-else class="size-3.5" />
      </UiIconButton>
    </div>
    <button
      v-if="hasKey"
      type="button"
      :disabled="disabled"
      class="w-fit rounded-sm text-caption text-fg-secondary underline-offset-2 hover:text-fg-primary hover:underline focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
      @click="secret = secret === null ? undefined : null"
    >
      {{ secret === null ? t("common.cancel") : t("settings.credentials.remove") }}
    </button>
    <p v-if="secret === null" class="text-caption text-fg-secondary">
      {{ t("settings.credentials.removalPending") }}
    </p>
    <p v-if="loadError" class="text-caption text-fg-danger" role="alert">
      {{ t("settings.credentials.loadFailed") }}: {{ loadError }}
    </p>
  </div>
</template>
