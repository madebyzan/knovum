<script setup lang="ts">
import { useI18n } from "vue-i18n";

import AiExecutionSourceLabel from "../../../../foundation/ai/AiExecutionSourceLabel.vue";
import type { GroundedDocumentMetadata } from "../../knowledge-derivations/document/metadata";
import SpaceDerivationSources from "../../knowledge-derivations/presentation/SpaceDerivationSources.vue";

defineProps<{ metadata: GroundedDocumentMetadata }>();
const { d, t } = useI18n();
</script>

<template>
  <div class="space-y-4">
    <div class="space-y-1 text-caption text-fg-secondary">
      <p>
        <AiExecutionSourceLabel :source="metadata.executionSource" />
      </p>
      <p class="text-fg-tertiary">{{ d(metadata.createdAt, "short") }}</p>
    </div>
    <SpaceDerivationSources :sources="metadata.sources" />
    <div>
      <h4 class="mb-1.5 text-caption-sm font-medium text-fg-tertiary">
        {{ t("spaces.creation.shared.citations", { count: metadata.citations.length }) }}
      </h4>
      <p
        v-for="citation in metadata.citations"
        :key="`${citation.sourceRevision}:${citation.quote}`"
        class="mb-2 border-l border-stroke-divider pl-2 text-caption text-fg-secondary"
      >
        <span class="font-medium text-fg-primary">{{ citation.nodeTitle }}</span> —
        {{ citation.quote }}
      </p>
    </div>
  </div>
</template>
