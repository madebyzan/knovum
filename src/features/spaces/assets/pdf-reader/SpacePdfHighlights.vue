<script setup lang="ts">
import { Image as ImageIcon, LocateFixed, StickyNote } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { PdfProvenanceLink } from "./use-pdf-source-links";

defineProps<{
  links: readonly PdfProvenanceLink[];
  currentPage: number;
}>();

const emit = defineEmits<{
  jumpPage: [link: PdfProvenanceLink];
  revealCanvas: [targetNodeId: string];
}>();

const { t } = useI18n();
</script>

<template>
  <div v-if="links.length" class="space-y-1.5 p-2">
    <div
      v-for="link in links"
      :key="link.id"
      class="group rounded-lg border border-stroke-divider/70 bg-surface-raised p-2 transition-colors hover:border-stroke-strong"
      :class="link.page === currentPage && 'border-surface-accent/50'"
    >
      <div class="flex items-center justify-between gap-1">
        <button
          type="button"
          class="flex min-w-0 flex-1 items-center gap-1.5 text-left"
          @click="emit('jumpPage', link)"
        >
          <ImageIcon v-if="link.targetKind === 'image'" class="size-3 shrink-0 text-fg-accent" />
          <StickyNote v-else class="size-3 shrink-0 text-amber-500" />
          <span class="truncate text-caption-sm font-medium text-fg-primary">{{ link.targetTitle }}</span>
        </button>
        <span class="shrink-0 text-[10px] text-fg-tertiary tabular-nums">p.{{ link.page }}</span>
        <UiIconButton
          :label="t('spaces.canvas.revealInCanvas')"
          size="sm"
          tooltip
          @click="emit('revealCanvas', link.targetNodeId)"
        >
          <LocateFixed class="size-3" />
        </UiIconButton>
      </div>
      <button
        v-if="link.quote && link.targetKind !== 'image'"
        type="button"
        class="mt-1 line-clamp-3 w-full border-l-2 border-amber-400/75 pl-2 text-left text-caption-sm leading-relaxed text-fg-secondary"
        @click="emit('jumpPage', link)"
      >
        {{ link.quote }}
      </button>
    </div>
  </div>
  <p v-else class="p-6 text-center text-caption-sm text-fg-tertiary">
    {{ t("spaces.conversation.sourcePeek.noText") }}
  </p>
</template>
