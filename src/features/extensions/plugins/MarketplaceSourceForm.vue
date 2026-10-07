<script setup lang="ts">
import { reactive } from "vue";

import type { MarketplaceSource } from "../../../shared/contracts/generated/MarketplaceSource";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
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
  <form class="space-y-4" @submit.prevent="submit">
    <div class="space-y-1.5">
      <span class="text-caption text-fg-secondary">{{ $t("extensions.marketplaceSource") }}</span>
      <UiSelect
        v-model="form.type"
        :label="$t('extensions.marketplaceSource')"
        data-testid="marketplace-source-type"
        :disabled="busy"
        :options="[
          { value: 'https', label: 'HTTPS' },
          { value: 'directory', label: $t('extensions.localDirectory') },
          { value: 'git', label: 'HTTPS Git' },
        ]"
      />
    </div>
    <label class="block space-y-1.5 text-caption text-fg-secondary"
      >{{ $t("extensions.location")
      }}<UiInput :disabled="busy" v-model="form.location" data-testid="marketplace-location"
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
      size="sm"
      :disabled="busy || !form.location.trim()"
      data-testid="add-marketplace"
      >{{ $t("extensions.addSource") }}</UiButton
    >
  </form>
</template>
