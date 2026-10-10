<script setup lang="ts">
import type { McpEndpoint } from "../../../../shared/contracts/generated/McpEndpoint";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
const endpoint = defineModel<Extract<McpEndpoint, { type: "streamable_http" }>>({ required: true });
defineProps<{ disabled?: boolean }>();
</script>
<template>
  <div class="space-y-4">
    <label class="block space-y-1.5 text-caption text-fg-secondary">
      {{ $t("extensions.connections.configuration.serverUrl") }}
      <UiInput
        v-model="endpoint.url"
        :disabled="disabled"
        placeholder="https://example.com/mcp"
        data-testid="mcp-endpoint-url"
        required
      />
    </label>
    <label class="flex items-center gap-2 text-caption">
      <UiCheckbox
        v-model="endpoint.allowLocalNetwork"
        :disabled="disabled"
        :label="$t('extensions.connections.allowLocalNetwork')"
      />
      {{ $t("extensions.connections.allowLocalNetwork") }}
    </label>
  </div>
</template>
