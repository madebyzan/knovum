<script setup lang="ts">
import { FolderOpen } from "@lucide/vue";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, shallowRef, useId } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { McpEndpoint } from "../../../../shared/contracts/generated/McpEndpoint";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import McpArguments from "./McpArguments.vue";
type StdioEndpoint = Extract<McpEndpoint, { type: "stdio" }>;
const endpoint = defineModel<StdioEndpoint>({ required: true });
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
    if (typeof path === "string" && endpoint.value === current)
      endpoint.value = { ...current, [target]: path };
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    choosing.value = false;
  }
}
const cwd = computed({
  get: () => endpoint.value.cwd ?? "",
  set: (value: string) => {
    endpoint.value = { ...endpoint.value, cwd: value || null };
  },
});
</script>
<template>
  <div class="space-y-4">
    <div class="space-y-1.5 text-caption text-fg-secondary">
      <label :for="executableInputId">{{ $t("extensions.endpoint.process.executable.label") }}</label>
      <div class="flex gap-2">
        <UiInput
          :id="executableInputId"
          :disabled="disabled || choosing"
          v-model="endpoint.executable"
          data-testid="mcp-executable"
          required
        /><UiIconButton
          :label="$t('extensions.endpoint.process.executable.choose')"
          :disabled="disabled || choosing"
          @click="choosePath('executable')"
          ><FolderOpen class="size-4"
        /></UiIconButton>
      </div>
    </div>
    <McpArguments v-model="endpoint.args" :disabled="disabled" />
    <div class="space-y-1.5 text-caption text-fg-secondary">
      <label :for="directoryInputId">{{ $t("extensions.endpoint.process.directory.label") }}</label>
      <div class="flex gap-2">
        <UiInput
          :id="directoryInputId"
          :disabled="disabled || choosing"
          v-model="cwd"
          data-testid="mcp-working-directory"
        /><UiIconButton
          :label="$t('extensions.endpoint.process.directory.choose')"
          :disabled="disabled || choosing"
          @click="choosePath('cwd')"
          ><FolderOpen class="size-4"
        /></UiIconButton>
      </div>
    </div>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.endpoint.process.authorizationHint") }}</p>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
  </div>
</template>
