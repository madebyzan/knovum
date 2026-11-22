<script setup lang="ts">
import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import type { DocumentRenderBlock } from "../document-block-projection";

defineProps<{
  block: DocumentRenderBlock;
  showSlideHeader: boolean;
  showSectionLabel: boolean;
  active: boolean;
  exactQuote?: string | undefined;
}>();
</script>

<template>
  <div
    :data-chunk-index="block.chunkIndex"
    class="space-y-2 rounded-lg transition-colors"
    :class="active && '-mx-2.5 border-l-2 border-stroke-accent bg-surface-accent-subtle/45 px-2.5 py-2'"
  >
    <div
      v-if="showSlideHeader"
      data-reader-ignore-selection
      class="border-b border-stroke-divider pb-1 text-caption-sm font-semibold tracking-wide text-fg-tertiary uppercase select-none"
    >
      Slide {{ block.slideNumber }}
    </div>
    <div
      v-else-if="showSectionLabel"
      data-reader-ignore-selection
      class="text-caption-sm font-medium text-fg-tertiary select-none"
    >
      {{ block.sectionLabel }}
    </div>

    <h3 v-if="block.kind === 'heading'" class="pt-2 text-title leading-snug font-semibold text-fg-primary">
      {{ block.text }}
    </h3>

    <div
      v-else-if="block.kind === 'table' && block.tableRows?.length"
      class="overflow-x-auto rounded-lg border border-stroke-divider"
    >
      <table class="w-full border-collapse text-left text-caption">
        <tbody class="divide-y divide-stroke-divider/70">
          <tr
            v-for="(row, rIdx) in block.tableRows"
            :key="rIdx"
            :class="rIdx === 0 ? 'bg-surface-control font-medium' : 'hover:bg-surface-control/30'"
          >
            <td
              v-for="(cell, cIdx) in row"
              :key="cIdx"
              class="border-r border-stroke-divider/60 px-2.5 py-1.5 text-fg-primary last:border-r-0"
            >
              {{ cell }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div
      v-else-if="block.kind === 'code' && block.codeLines?.length"
      class="overflow-x-auto rounded-lg border border-stroke-divider bg-surface-control/60 p-2 font-mono text-caption"
    >
      <div
        v-for="line in block.codeLines"
        :key="line.number"
        :data-code-line="line.number"
        class="flex gap-3 px-1 leading-relaxed"
      >
        <span data-reader-ignore-selection class="w-8 shrink-0 text-right text-fg-tertiary select-none">{{
          line.number
        }}</span>
        <span class="whitespace-pre text-fg-primary">{{ line.text || " " }}</span>
      </div>
    </div>

    <SpaceTextQuote v-else :text="block.text" :exact="exactQuote" />
  </div>
</template>
