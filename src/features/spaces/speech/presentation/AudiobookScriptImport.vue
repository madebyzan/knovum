<script setup lang="ts">
import { FileUp } from "@lucide/vue";
import { computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { ImportedAudiobookScript } from "../audiobook-script";
import { useAudiobookScriptImport } from "../use-audiobook-script-import";

const props = defineProps<{ node: SpaceCanvasNode; disabled: boolean; initialize: boolean }>();
const emit = defineEmits<{
  imported: [script: ImportedAudiobookScript, initialize: boolean];
  busy: [value: boolean];
}>();
const { t } = useI18n();
const {
  cancel,
  error,
  importing,
  importScript: runImport,
} = useAudiobookScriptImport(computed(() => props.node));

async function importScript(initialize = false) {
  const script = await runImport();
  if (script) emit("imported", script, initialize);
}
watch(importing, (value) => emit("busy", value), { immediate: true });
onMounted(() => {
  if (props.initialize) void importScript(true);
});
</script>

<template>
  <section class="space-y-1">
    <div class="flex items-center gap-2">
      <UiButton size="sm" variant="ghost" :disabled="disabled || importing" @click="importScript()">
        <FileUp class="size-3.5" />{{ t("spaces.speech.project.importManuscript") }}
      </UiButton>
    </div>
    <UiButton v-if="importing" size="sm" @click="cancel">{{ t("common.cancel") }}</UiButton>
    <p v-if="importing" class="text-caption text-fg-secondary" role="status">{{ t("common.loading") }}</p>
    <p v-if="error" class="text-caption-sm text-fg-danger">{{ error }}</p>
  </section>
</template>
