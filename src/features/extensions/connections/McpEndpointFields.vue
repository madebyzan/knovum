<script setup lang="ts">
import { computed } from "vue";

import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
const endpoint = defineModel<McpEndpoint>({ required: true });
defineProps<{ disabled?: boolean }>();
function setType(type: McpEndpoint["type"]) {
  endpoint.value =
    type === "stdio"
      ? { type: "stdio", executable: "", args: [], cwd: null, environment: {} }
      : { type: "streamable_http", url: "", headers: {}, allowLocalNetwork: false };
}
const transport = computed({ get: () => endpoint.value.type, set: setType });
const args = computed({
  get: () => (endpoint.value.type === "stdio" ? endpoint.value.args.join("\n") : ""),
  set: (value: string) => {
    if (endpoint.value.type === "stdio")
      endpoint.value = { ...endpoint.value, args: value ? value.split("\n") : [] };
  },
});
const cwd = computed({
  get: () => (endpoint.value.type === "stdio" ? (endpoint.value.cwd ?? "") : ""),
  set: (value: string) => {
    if (endpoint.value.type === "stdio") endpoint.value = { ...endpoint.value, cwd: value || null };
  },
});
</script>

<template>
  <div class="space-y-4">
    <div class="space-y-1.5">
      <span class="text-caption text-fg-secondary">{{ $t("extensions.transport") }}</span
      ><UiSelect
        v-model="transport"
        :disabled="disabled"
        :label="$t('extensions.transport')"
        :options="[
          { value: 'streamable_http', label: 'Streamable HTTP' },
          { value: 'stdio', label: 'stdio' },
        ]"
      />
    </div>
    <template v-if="endpoint.type === 'streamable_http'">
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >URL<UiInput :disabled="disabled" v-model="endpoint.url" data-testid="mcp-endpoint-url"
      /></label>
      <label class="flex items-center gap-2 text-caption"
        ><UiCheckbox
          v-model="endpoint.allowLocalNetwork"
          :disabled="disabled"
          :label="$t('extensions.allowLocalNetwork')"
        />{{ $t("extensions.allowLocalNetwork") }}</label
      >
    </template>
    <template v-else>
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.installedExecutable")
        }}<UiInput :disabled="disabled" v-model="endpoint.executable" data-testid="mcp-executable"
      /></label>
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.argumentsPerLine") }}<UiTextarea :disabled="disabled" v-model="args"
      /></label>
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.workingDirectory") }}<UiInput :disabled="disabled" v-model="cwd"
      /></label>
      <p class="text-caption text-fg-warning">{{ $t("extensions.explicitProcessAuthorization") }}</p>
    </template>
  </div>
</template>
