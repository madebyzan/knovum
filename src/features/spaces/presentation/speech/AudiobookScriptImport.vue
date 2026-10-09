<script setup lang="ts">
import { FileUp } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { ImportedAudiobookScript } from "../../speech/audiobook-script";
import { useAudiobookScriptImport } from "../../speech/use-audiobook-script-import";

const props = defineProps<{ node: SpaceCanvasNode; disabled: boolean }>();
const emit = defineEmits<{ imported: [script: ImportedAudiobookScript] }>();
const { t } = useI18n();
const {
  cancel,
  error,
  importing,
  importScript: runImport,
} = useAudiobookScriptImport(computed(() => props.node));

async function importScript() {
  const script = await runImport();
  if (script) emit("imported", script);
}
</script>

<template>
  <section class="space-y-1 rounded-md border border-stroke-divider p-2">
    <div class="flex items-center justify-between gap-2">
      <p class="text-caption-sm text-fg-secondary">{{ props.node.data.title }}</p>
      <UiButton size="sm" variant="ghost" :disabled="disabled || importing" @click="importScript">
        <FileUp class="size-3.5" />{{ t("spaces.speech.project.readManuscript") }}
      </UiButton>
    </div>
    <UiButton v-if="importing" size="sm" @click="cancel">{{ t("common.cancel") }}</UiButton>
    <p v-if="error" class="text-danger text-caption-sm">{{ error }}</p>
  </section>
</template>
