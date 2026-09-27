<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { CapturedWorkflowRun, SavedCapturedWorkflow } from "../../workflows/captured-workflow-ipc";
import { workflowStateLabelKey } from "../../workflows/workflow-state-label";

defineProps<{
  entries: CapturedWorkflowRun[];
  workflows: SavedCapturedWorkflow[];
  activeRunId: string;
  activeRunState: string;
}>();
const emit = defineEmits<{ select: [entry: CapturedWorkflowRun] }>();
const { t } = useI18n();
</script>

<template>
  <section v-if="entries.length" class="space-y-2 border-t border-stroke-divider pt-3">
    <h3 class="text-caption font-semibold text-fg-secondary">
      {{ t("spaces.creation.workflow.run.recent") }}
    </h3>
    <UiButton
      v-for="entry in entries"
      :key="entry.run.id"
      variant="ghost"
      class="h-auto w-full min-w-0 justify-start py-2 text-left"
      @click="emit('select', entry)"
    >
      <span class="min-w-0 flex-1 truncate">{{
        workflows.find((item) => item.workflowId === entry.workflowId)?.name ??
        t("spaces.creation.workflow.title")
      }}</span>
      <span class="text-caption-sm text-fg-tertiary">{{
        t(workflowStateLabelKey(entry.run.id === activeRunId ? activeRunState : entry.run.state))
      }}</span>
    </UiButton>
  </section>
</template>
