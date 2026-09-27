<script setup lang="ts">
import { CircleAlert, ExternalLink } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import type { DerivationSource } from "../../contracts/generated/DerivationSource";

const props = defineProps<{ sources: readonly DerivationSource[] }>();
const { t } = useI18n();
const hasChangedSource = computed(() => props.sources.some((source) => !source.current));
</script>

<template>
  <section v-if="sources.length">
    <h4 class="mb-1.5 text-caption-sm font-medium text-fg-tertiary">
      {{ t("spaces.creation.shared.source.knowledge") }}
    </h4>
    <div
      v-for="source in sources"
      :key="`${source.role}:${source.sourceIdentity}`"
      class="flex items-center gap-2 py-1"
    >
      <CircleAlert
        v-if="!source.current"
        class="size-3 shrink-0 text-fg-warning"
        :aria-label="t('spaces.creation.shared.source.changed')"
      />
      <span class="min-w-0 flex-1 truncate text-caption text-fg-secondary">{{ source.title }}</span>
      <UiIconButton
        v-if="source.nodeId !== null"
        :label="t('spaces.inspector.openSource')"
        size="sm"
        @click="source.nodeId !== null && selectAndRevealSpaceNode(source.nodeId)"
      >
        <ExternalLink class="size-3" />
      </UiIconButton>
    </div>
    <p v-if="hasChangedSource" class="mt-1 text-caption-sm text-fg-warning">
      {{ t("spaces.creation.shared.source.changed") }}
    </p>
  </section>
</template>
