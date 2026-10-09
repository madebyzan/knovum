<script setup lang="ts">
import { FolderOpen } from "@lucide/vue";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, shallowRef, useId } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
const endpoint = defineModel<McpEndpoint>({ required: true });
const props = defineProps<{ disabled?: boolean }>();
const executableInputId = useId();
const directoryInputId = useId();
const choosing = shallowRef(false);
const error = shallowRef("");
async function choosePath(target: "executable" | "cwd") {
  if (props.disabled || choosing.value) return;
  const current = endpoint.value;
  choosing.value = true;
  error.value = "";
  try {
    const path = await open({ directory: target === "cwd", multiple: false });
    if (typeof path === "string" && endpoint.value === current && current.type === "stdio")
      endpoint.value = { ...current, [target]: path };
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    choosing.value = false;
  }
}
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
      <span class="text-caption text-fg-secondary">{{ $t("extensions.connections.transport") }}</span
      ><UiSelect
        v-model="transport"
        :disabled="disabled"
        :label="$t('extensions.connections.transport')"
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
          :label="$t('extensions.connections.allowLocalNetwork')"
        />{{ $t("extensions.connections.allowLocalNetwork") }}</label
      >
    </template>
    <template v-else>
      <div class="space-y-1.5 text-caption text-fg-secondary">
        <label :for="executableInputId">{{ $t("extensions.connectors.process.executable") }}</label>
        <div class="flex gap-2">
          <UiInput
            :id="executableInputId"
            :disabled="disabled || choosing"
            v-model="endpoint.executable"
            data-testid="mcp-executable"
          /><UiIconButton
            :label="$t('extensions.connectors.process.chooseExecutable')"
            :disabled="disabled || choosing"
            @click="choosePath('executable')"
            ><FolderOpen class="size-4"
          /></UiIconButton>
        </div>
      </div>
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.connectors.process.arguments") }}<UiTextarea :disabled="disabled" v-model="args"
      /></label>
      <div class="space-y-1.5 text-caption text-fg-secondary">
        <label :for="directoryInputId">{{ $t("extensions.connectors.process.workingDirectory") }}</label>
        <div class="flex gap-2">
          <UiInput :id="directoryInputId" :disabled="disabled || choosing" v-model="cwd" /><UiIconButton
            :label="$t('extensions.connectors.process.chooseDirectory')"
            :disabled="disabled || choosing"
            @click="choosePath('cwd')"
            ><FolderOpen class="size-4"
          /></UiIconButton>
        </div>
      </div>
      <p class="text-caption text-fg-warning">{{ $t("extensions.connectors.process.authorizationHint") }}</p>
    </template>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
  </div>
</template>
