<script setup lang="ts">
import { computed } from "vue";

import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";
const props = defineProps<{ plugin: PluginInstallation }>();
const unsupportedComponents = computed(() =>
  props.plugin.revision.components.filter((component) => !component.supported),
);
const source = computed(() =>
  props.plugin.source.type === "directory" || props.plugin.source.type === "zip"
    ? props.plugin.source.path
    : props.plugin.source.url,
);
const author = computed(() => {
  const declared = props.plugin.revision.manifest.author;
  if (declared && typeof declared === "object" && "name" in declared && typeof declared.name === "string")
    return declared.name;
  return undefined;
});
</script>
<template>
  <section class="space-y-3">
    <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
      {{ $t("extensions.overview.information") }}
    </h3>
    <dl class="grid grid-cols-[auto_1fr] gap-x-6 gap-y-2 text-body">
      <template v-if="author"
        ><dt class="text-fg-secondary">{{ $t("extensions.overview.author") }}</dt>
        <dd>{{ author }}</dd></template
      >
      <template v-if="plugin.revision.manifest.version"
        ><dt class="text-fg-secondary">{{ $t("extensions.overview.version") }}</dt>
        <dd>{{ plugin.revision.manifest.version }}</dd></template
      >
      <dt class="text-fg-secondary">{{ $t("extensions.plugins.source.label") }}</dt>
      <dd class="break-all">{{ source }}</dd>
      <dt class="text-fg-secondary">{{ $t("extensions.plugins.source.revision") }}</dt>
      <dd class="font-mono text-caption break-all">{{ plugin.revision.sourceRevision }}</dd>
      <template v-if="plugin.revision.manifest.homepage"
        ><dt class="text-fg-secondary">{{ $t("extensions.overview.homepage") }}</dt>
        <dd class="break-all">{{ plugin.revision.manifest.homepage }}</dd></template
      >
      <template v-if="plugin.revision.manifest.repository"
        ><dt class="text-fg-secondary">{{ $t("extensions.overview.repository") }}</dt>
        <dd class="break-all">{{ plugin.revision.manifest.repository }}</dd></template
      >
      <template v-if="plugin.revision.manifest.license"
        ><dt class="text-fg-secondary">{{ $t("extensions.overview.license") }}</dt>
        <dd>{{ plugin.revision.manifest.license }}</dd></template
      >
      <template v-if="plugin.publication"
        ><dt class="text-fg-secondary">{{ $t("extensions.plugins.verification.unverified") }}</dt>
        <dd class="break-all">{{ plugin.publication.publisherId }}</dd></template
      >
    </dl>
    <p
      v-for="component in unsupportedComponents"
      :key="`${component.kind}:${component.name}`"
      class="text-caption text-fg-warning"
    >
      {{ component.name }}: {{ component.diagnostic }}
    </p>
  </section>
</template>
