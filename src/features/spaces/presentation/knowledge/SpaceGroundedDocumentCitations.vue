<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { GroundedDocumentMetadata } from "../../knowledge-derivations/document/metadata";
import SpaceDerivationSources from "../derivations/SpaceDerivationSources.vue";

defineProps<{ metadata: GroundedDocumentMetadata }>();
const { d, t } = useI18n();
</script>

<template>
  <div class="space-y-4">
    <div class="space-y-1 text-caption-md text-fg-secondary">
      <p>
        {{
          t("spaces.creation.shared.generatedWith", { provider: metadata.provider, model: metadata.model })
        }}
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
        class="mb-2 border-l border-stroke-divider pl-2 text-caption-md text-fg-secondary"
      >
        <span class="font-medium text-fg-primary">{{ citation.nodeTitle }}</span> —
        {{ citation.quote }}
      </p>
    </div>
  </div>
</template>
