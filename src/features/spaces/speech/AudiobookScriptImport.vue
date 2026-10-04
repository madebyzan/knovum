<script setup lang="ts">
import { FileUp } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import { chooseAudiobookScript, type ImportedAudiobookScript } from "./audiobook-script-import";

defineProps<{ disabled: boolean }>();
const emit = defineEmits<{ imported: [script: ImportedAudiobookScript] }>();
const { t } = useI18n();
const importing = shallowRef(false);
const error = shallowRef<string>();

async function importScript() {
  if (importing.value) return;
  importing.value = true;
  try {
    const script = await chooseAudiobookScript();
    if (script) emit("imported", script);
    error.value = undefined;
  } catch {
    error.value = t("spaces.speech.audiobookImportFailed");
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <section class="space-y-1 rounded-md border border-stroke-divider p-2">
    <div class="flex items-center justify-between gap-2">
      <p class="text-caption-sm text-fg-secondary">{{ t("spaces.speech.audiobookImportHint") }}</p>
      <UiButton size="sm" variant="ghost" :disabled="disabled || importing" @click="importScript">
        <FileUp class="size-3.5" />{{ t("spaces.speech.importAudiobookScript") }}
      </UiButton>
    </div>
    <p v-if="error" class="text-danger text-caption-sm">{{ error }}</p>
  </section>
</template>
