<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { WorkflowVersion } from "../../workflows/workflow-client";
import { diffWorkflowVersions } from "../../workflows/workflow-version-diff";

const props = defineProps<{ previous: WorkflowVersion; current: WorkflowVersion }>();
const { t } = useI18n();
const diff = computed(() => diffWorkflowVersions(props.previous.definition, props.current.definition));
const groups = computed(
  () =>
    [
      ["addedNodes", t("spaces.assistant.diffAddedSteps"), diff.value.addedNodes],
      ["removedNodes", t("spaces.assistant.diffRemovedSteps"), diff.value.removedNodes],
      ["changedNodes", t("spaces.assistant.diffChangedSteps"), diff.value.changedNodes],
      ["addedEdges", t("spaces.assistant.diffAddedConnections"), diff.value.addedEdges],
      ["removedEdges", t("spaces.assistant.diffRemovedConnections"), diff.value.removedEdges],
      ["policyChanges", t("spaces.assistant.diffPolicyChanges"), diff.value.policyChanges],
    ] as const,
);
const empty = computed(() => groups.value.every(([, , values]) => values.length === 0));
</script>

<template>
  <div class="mt-2 rounded-panel border border-border bg-raised p-2.5">
    <p class="text-caption text-foreground-secondary">v{{ previous.version }} → v{{ current.version }}</p>
    <p v-if="empty" class="mt-2 text-caption text-foreground-secondary">
      {{ t("spaces.assistant.diffEmpty") }}
    </p>
    <template v-for="[key, label, values] in groups" :key="key">
      <section v-if="values.length" class="mt-2">
        <h4 class="text-body text-caption font-medium">{{ label }} · {{ values.length }}</h4>
        <ul class="mt-1 space-y-0.5 text-micro text-foreground-secondary">
          <li v-for="value in values" :key="value">{{ value }}</li>
        </ul>
      </section>
    </template>
  </div>
</template>
