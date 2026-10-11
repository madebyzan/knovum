<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { useId } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { EndpointField } from "../endpoint-fields";
const fields = defineModel<EndpointField[]>({ required: true });
defineProps<{ transport: "stdio" | "streamable_http"; disabled: boolean }>();
const headingId = useId();
</script>
<template>
  <section class="space-y-2" :aria-labelledby="headingId" data-testid="mcp-public-fields">
    <header class="flex items-center justify-between gap-2">
      <h3 :id="headingId" class="text-label font-medium">
        {{ $t(`extensions.endpoint.configuration.${transport === "stdio" ? "environment" : "headers"}`) }}
      </h3>
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled"
        data-testid="add-endpoint-field"
        @click="fields.push({ name: '', value: '' })"
      >
        <Plus class="size-3.5" />{{
          $t(`extensions.endpoint.configuration.${transport === "stdio" ? "addVariable" : "addHeader"}`)
        }}
      </UiButton>
    </header>
    <div v-for="(field, index) in fields" :key="index" class="flex items-center gap-2">
      <UiInput
        v-model="field.name"
        :disabled="disabled"
        class="min-w-0 flex-1 font-mono text-caption"
        :placeholder="$t('extensions.endpoint.configuration.fieldName')"
        :aria-label="$t('extensions.endpoint.configuration.fieldNameAt', { index: index + 1 })"
        :data-testid="`endpoint-field-name-${index}`"
      />
      <UiInput
        v-model="field.value"
        :disabled="disabled"
        class="min-w-0 flex-1 font-mono text-caption"
        :placeholder="$t('extensions.endpoint.configuration.fieldValue')"
        :aria-label="$t('extensions.endpoint.configuration.fieldValueAt', { index: index + 1 })"
        :data-testid="`endpoint-field-value-${index}`"
      />
      <UiIconButton
        size="sm"
        :disabled="disabled"
        :label="$t('extensions.endpoint.configuration.removeField', { index: index + 1 })"
        @click="fields.splice(index, 1)"
        ><Trash2 class="size-3.5"
      /></UiIconButton>
    </div>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.endpoint.configuration.secretsHint") }}</p>
  </section>
</template>
