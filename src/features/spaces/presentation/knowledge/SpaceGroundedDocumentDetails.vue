<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { GroundedDocumentMetadata } from "../../knowledge-derivations/document/metadata";
import SpaceDerivationSources from "../derivations/SpaceDerivationSources.vue";

defineProps<{ metadata: GroundedDocumentMetadata }>();
const { t } = useI18n();
</script>

<template>
  <div class="space-y-4">
    <div class="space-y-1 text-caption text-foreground-secondary">
      <p>
        {{ t("spaces.creation.generatedWith", { provider: metadata.provider, model: metadata.model }) }}
      </p>
      <p class="text-foreground-tertiary">{{ new Date(metadata.createdAt).toLocaleString() }}</p>
    </div>
    <SpaceDerivationSources :sources="metadata.sources" />
    <div>
      <h4 class="mb-1.5 text-micro font-medium text-foreground-tertiary">
        {{ t("spaces.creation.citations", { count: metadata.citations.length }) }}
      </h4>
      <p
        v-for="citation in metadata.citations"
        :key="`${citation.sourceRevision}:${citation.quote}`"
        class="mb-2 border-l border-divider pl-2 text-caption text-foreground-secondary"
      >
        <span class="font-medium text-foreground">{{ citation.nodeTitle }}</span> — {{ citation.quote }}
      </p>
    </div>
  </div>
</template>
