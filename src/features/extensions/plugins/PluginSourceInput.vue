<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { computed, reactive, ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { PluginSource } from "../../../shared/contracts/generated/PluginSource";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
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
  <form class="space-y-4" @submit.prevent="!busy && form.location.trim() && emit('inspect', source)">
    <div class="space-y-1.5">
      <span class="text-caption text-fg-secondary">{{ $t("extensions.source") }}</span>
      <UiSelect
        v-model="form.type"
        :label="$t('extensions.source')"
        data-testid="plugin-source-type"
        :disabled="busy"
        :options="[
          { value: 'directory', label: $t('extensions.localDirectory') },
          { value: 'zip', label: 'ZIP' },
          { value: 'https_zip', label: 'HTTPS ZIP' },
          { value: 'git', label: 'HTTPS Git' },
        ]"
      />
    </div>
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ $t("extensions.location")
      }}<UiInput :disabled="busy" v-model="form.location" data-testid="plugin-source-location"
    /></label>
    <UiButton
      v-if="form.type === 'directory' || form.type === 'zip'"
      size="sm"
      :disabled="busy"
      data-testid="choose-plugin-source"
      @click="choose"
      >{{ $t("extensions.chooseSource") }}</UiButton
    >
    <p v-if="pickerError" role="alert" class="text-caption text-fg-danger">{{ pickerError }}</p>
    <label v-if="form.type === 'https_zip'" class="block space-y-1.5 text-caption text-fg-secondary"
      >SHA-256<UiInput :disabled="busy" v-model="form.sha256"
    /></label>
    <template v-if="form.type === 'git'">
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.gitReference") }}<UiInput :disabled="busy" v-model="form.reference"
      /></label>
      <label class="block space-y-1.5 text-caption text-fg-secondary"
        >{{ $t("extensions.subdirectory") }}<UiInput :disabled="busy" v-model="form.subdirectory"
      /></label>
    </template>
    <UiButton
      variant="primary"
      type="submit"
      :disabled="props.busy || !form.location.trim()"
      data-testid="inspect-plugin"
      ><UiSpinner v-if="busy" class="size-3.5" />{{ $t("extensions.preview") }}</UiButton
    >
  </form>
</template>
