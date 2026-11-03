<script setup lang="ts">
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import type { EnrichmentJobState } from "../../contracts/generated/EnrichmentJobState";
import SpaceEnrichmentJobStatus from "./SpaceEnrichmentJobStatus.vue";

/** The text local recognition read from one picture, beside the picture it belongs to. */
defineProps<{ text: string | undefined; job?: EnrichmentJobState | undefined }>();
const emit = defineEmits<{ retried: [] }>();
</script>

<template>
  <div v-if="text || job" class="space-y-1">
    <h4 class="text-caption font-medium text-fg-tertiary">
      {{ $t("spaces.inspector.recognizedText") }}
    </h4>
    <UiExpandableText
      v-if="text"
      :text="text"
      :clamp-lines="4"
      preformatted
      class="text-caption leading-relaxed text-fg-secondary select-text"
    />
    <SpaceEnrichmentJobStatus :job="job" @retried="emit('retried')" />
  </div>
</template>
