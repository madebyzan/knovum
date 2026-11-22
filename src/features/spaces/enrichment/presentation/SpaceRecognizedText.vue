<script setup lang="ts">
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import type { EnrichmentJobState } from "../../contracts/generated/EnrichmentJobState";
import SpaceEnrichmentJobStatus from "./SpaceEnrichmentJobStatus.vue";

/** The text local recognition read from one picture, beside the picture it belongs to. */
defineProps<{ text: string | undefined; job?: EnrichmentJobState | undefined }>();
const emit = defineEmits<{ retried: [] }>();
</script>

<template>
  <div v-if="text || job" class="space-y-1.5">
    <h4 class="text-caption font-medium text-fg-tertiary">
      {{ $t("spaces.inspector.recognizedText") }}
    </h4>
    <div v-if="text" class="rounded-md bg-surface-control/45 px-2.5 py-2">
      <UiExpandableText
        :text="text"
        :clamp-lines="4"
        preformatted
        class="text-caption leading-relaxed text-fg-secondary select-text"
      />
    </div>
    <SpaceEnrichmentJobStatus :job="job" @retried="emit('retried')" />
  </div>
</template>
