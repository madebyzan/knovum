<script setup lang="ts">
import { computed } from "vue";

import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
const endpoint = defineModel<McpEndpoint>({ required: true });
function setType(event: Event) {
  const type = (event.target as HTMLSelectElement).value;
  endpoint.value =
    type === "stdio"
      ? { type: "stdio", executable: "", args: [], cwd: null, environment: {} }
      : { type: "streamable_http", url: "", headers: {}, allowLocalNetwork: false };
}
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
  <div class="space-y-2">
    <label class="flex items-center gap-2 text-caption"
      >{{ $t("extensions.transport")
      }}<select :value="endpoint.type" class="rounded-sm bg-surface-control p-1" @change="setType">
        <option value="streamable_http">Streamable HTTP</option>
        <option value="stdio">stdio</option>
      </select></label
    >
    <template v-if="endpoint.type === 'streamable_http'">
      <label class="block text-caption"
        >URL<UiInput v-model="endpoint.url" data-testid="mcp-endpoint-url"
      /></label>
      <label class="flex items-center gap-2 text-caption"
        ><input v-model="endpoint.allowLocalNetwork" type="checkbox" />{{
          $t("extensions.allowLocalNetwork")
        }}</label
      >
    </template>
    <template v-else>
      <label class="block text-caption"
        >{{ $t("extensions.installedExecutable")
        }}<UiInput v-model="endpoint.executable" data-testid="mcp-executable"
      /></label>
      <label class="block text-caption"
        >{{ $t("extensions.argumentsPerLine") }}<UiTextarea v-model="args"
      /></label>
      <label class="block text-caption"
        >{{ $t("extensions.workingDirectory") }}<UiInput v-model="cwd"
      /></label>
      <p class="text-caption text-fg-warning">{{ $t("extensions.explicitProcessAuthorization") }}</p>
    </template>
  </div>
</template>
