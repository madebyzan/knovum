<script setup lang="ts">
import { computed } from "vue";

import type { PluginInspection } from "../../../shared/contracts/generated/PluginInspection";
const props = defineProps<{ before: PluginInspection; after: PluginInspection }>();
const changes = computed(() => {
  const old = new Map(
    props.before.components.map((component) => [`${component.kind}:${component.name}`, component]),
  );
  const next = new Map(
    props.after.components.map((component) => [`${component.kind}:${component.name}`, component]),
  );
  return {
    added: [...next.keys()].filter((key) => !old.has(key)),
    removed: [...old.keys()].filter((key) => !next.has(key)),
    connectorsChanged: JSON.stringify(props.before.connectors) !== JSON.stringify(props.after.connectors),
  };
});
</script>

<template>
  <section class="space-y-2 text-caption" data-testid="plugin-revision-changes">
    <p>{{ $t("extensions.revisionChange") }}: {{ before.sourceRevision }} → {{ after.sourceRevision }}</p>
    <p v-if="changes.added.length">{{ $t("extensions.added") }}: {{ changes.added.join(", ") }}</p>
    <p v-if="changes.removed.length">{{ $t("extensions.removed") }}: {{ changes.removed.join(", ") }}</p>
    <div v-if="changes.connectorsChanged" class="space-y-1">
      <p>{{ $t("extensions.reviewConnectorChanges") }}</p>
      <pre class="max-h-48 overflow-auto whitespace-pre-wrap">{{
        JSON.stringify(before.connectors, null, 2)
      }}</pre>
      <pre class="max-h-48 overflow-auto whitespace-pre-wrap">{{
        JSON.stringify(after.connectors, null, 2)
      }}</pre>
    </div>
  </section>
</template>
