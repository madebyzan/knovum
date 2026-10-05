<script setup lang="ts">
import { FileUp } from "@lucide/vue";
import { onScopeDispose, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { ImportedAudiobookScript } from "../../speech/audiobook-script";
import { chooseAudiobookScript } from "../../speech/audiobook-script-import";
import { cancelSpeechGeneration } from "../../speech/ipc";

defineProps<{ disabled: boolean }>();
const emit = defineEmits<{ imported: [script: ImportedAudiobookScript] }>();
const { t } = useI18n();
const importing = shallowRef(false);
const error = shallowRef<string>();
const jobId = shallowRef<string>();
let disposed = false;
async function cancel() {
  const job = jobId.value;
  jobId.value = undefined;
  if (job) await cancelSpeechGeneration(job);
}
onScopeDispose(() => {
  disposed = true;
  void cancel().catch((cause) => {
    error.value = getErrorMessage(cause);
  });
});

async function importScript() {
  if (importing.value) return;
  importing.value = true;
  try {
    const job = crypto.randomUUID();
    jobId.value = job;
    const script = await chooseAudiobookScript(
      t("spaces.speech.defaultRole"),
      job,
      () => !disposed && jobId.value === job,
    );
    if (script && !disposed && jobId.value === job) emit("imported", script);
    error.value = undefined;
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    importing.value = false;
    jobId.value = undefined;
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
    <UiButton v-if="importing" size="sm" @click="cancel">{{ t("spaces.speech.cancel") }}</UiButton>
    <p v-if="error" class="text-danger text-caption-sm">{{ error }}</p>
  </section>
</template>
