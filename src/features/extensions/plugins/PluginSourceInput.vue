<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { computed, reactive, ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { PluginSource } from "../../../shared/contracts/generated/PluginSource";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
const props = defineProps<{ busy: boolean }>();
const emit = defineEmits<{ inspect: [source: PluginSource] }>();
const form = reactive({ type: "directory", location: "", reference: "HEAD", subdirectory: "", sha256: "" });
const source = computed<PluginSource>(() => {
  if (form.type === "https_zip")
    return { type: "https_zip", url: form.location, sha256: form.sha256 || null };
  if (form.type === "git")
    return { type: "git", url: form.location, reference: form.reference, subdirectory: form.subdirectory };
  return form.type === "zip"
    ? { type: "zip", path: form.location }
    : { type: "directory", path: form.location };
});
const pickerError = ref("");
async function choose() {
  try {
    const path = await open({
      directory: form.type === "directory",
      multiple: false,
      ...(form.type === "zip" && { filters: [{ name: "ZIP", extensions: ["zip"] }] }),
    });
    if (typeof path === "string") form.location = path;
    pickerError.value = "";
  } catch (error) {
    pickerError.value = getErrorMessage(error);
  }
}
</script>

<template>
  <form
    class="space-y-3 rounded-sm border border-stroke-divider p-3"
    @submit.prevent="emit('inspect', source)"
  >
    <label class="flex items-center gap-2 text-body"
      >{{ $t("extensions.source") }}
      <select
        v-model="form.type"
        data-testid="plugin-source-type"
        class="rounded-sm bg-surface-control px-2 py-1"
      >
        <option value="directory">{{ $t("extensions.localDirectory") }}</option>
        <option value="zip">ZIP</option>
        <option value="https_zip">HTTPS ZIP</option>
        <option value="git">HTTPS Git</option>
      </select>
    </label>
    <label class="block text-caption"
      >{{ $t("extensions.location") }}<UiInput v-model="form.location" data-testid="plugin-source-location"
    /></label>
    <UiButton
      v-if="form.type === 'directory' || form.type === 'zip'"
      size="sm"
      :disabled="busy"
      @click="choose"
      >{{ $t("extensions.chooseSource") }}</UiButton
    >
    <p v-if="pickerError" role="alert" class="text-caption text-fg-danger">{{ pickerError }}</p>
    <label v-if="form.type === 'https_zip'" class="block text-caption"
      >SHA-256<UiInput v-model="form.sha256"
    /></label>
    <template v-if="form.type === 'git'">
      <label class="block text-caption"
        >{{ $t("extensions.gitReference") }}<UiInput v-model="form.reference"
      /></label>
      <label class="block text-caption"
        >{{ $t("extensions.subdirectory") }}<UiInput v-model="form.subdirectory"
      /></label>
    </template>
    <UiButton type="submit" :disabled="props.busy || !form.location" data-testid="inspect-plugin">{{
      $t("extensions.preview")
    }}</UiButton>
  </form>
</template>
