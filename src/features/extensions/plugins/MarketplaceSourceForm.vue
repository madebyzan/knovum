<script setup lang="ts">
import { reactive } from "vue";

import type { MarketplaceSource } from "../../../shared/contracts/generated/MarketplaceSource";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
defineProps<{ busy: boolean }>();
const emit = defineEmits<{ add: [source: MarketplaceSource] }>();
const form = reactive({ type: "https", location: "", reference: "HEAD", subdirectory: "" });
function submit() {
  if (form.type === "git")
    emit("add", {
      type: "git",
      url: form.location,
      reference: form.reference,
      subdirectory: form.subdirectory,
    });
  else if (form.type === "directory") emit("add", { type: "directory", path: form.location });
  else emit("add", { type: "https", url: form.location });
}
</script>

<template>
  <form class="space-y-2" @submit.prevent="submit">
    <label class="flex items-center gap-2 text-caption"
      >{{ $t("extensions.marketplaceSource") }}
      <select
        data-testid="marketplace-source-type"
        v-model="form.type"
        class="rounded-sm bg-surface-control px-2 py-1"
      >
        <option value="https">HTTPS</option>
        <option value="directory">{{ $t("extensions.localDirectory") }}</option>
        <option value="git">HTTPS Git</option>
      </select>
    </label>
    <label class="block text-caption"
      >{{ $t("extensions.location") }}<UiInput v-model="form.location" data-testid="marketplace-location"
    /></label>
    <template v-if="form.type === 'git'">
      <label class="block text-caption"
        >{{ $t("extensions.gitReference") }}<UiInput v-model="form.reference"
      /></label>
      <label class="block text-caption"
        >{{ $t("extensions.subdirectory") }}<UiInput v-model="form.subdirectory"
      /></label>
    </template>
    <UiButton type="submit" size="sm" :disabled="busy || !form.location" data-testid="add-marketplace">{{
      $t("extensions.addSource")
    }}</UiButton>
  </form>
</template>
