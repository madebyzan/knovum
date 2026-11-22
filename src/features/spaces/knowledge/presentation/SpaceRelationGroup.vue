<script setup lang="ts">
import { ChevronDown, LocateFixed, Pencil, Trash2 } from "@lucide/vue";
import { camelCase } from "es-toolkit";
import { AccordionContent, AccordionItem, AccordionTrigger } from "reka-ui";
import type { Component } from "vue";
import { useI18n } from "vue-i18n";

import { nodeIcon } from "../../canvas/presentation/node-icon";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import SpaceRelationEvidenceEditor from "./SpaceRelationEvidenceEditor.vue";

interface SpaceRelationNeighbor {
  relation: SpaceRelation;
  neighbor: SpaceCanvasNode;
  thumbnail: string;
}

defineProps<{
  relations: SpaceRelationNeighbor[];
  icon: Component;
  labelKey: string;
}>();

const emit = defineEmits<{
  reveal: [neighborId: string];
  editRelation: [relationId: string];
  removeRelation: [relationId: string, neighborTitle: string];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="space-y-1">
    <h4 class="flex items-center gap-1.5 text-caption-sm font-medium text-fg-tertiary">
      <component :is="icon" class="size-3" :stroke-width="1.6" />
      {{ t(labelKey, { count: relations.length }) }}
    </h4>
    <AccordionItem
      v-for="item in relations"
      :key="item.relation.id"
      :value="item.relation.id"
      class="-mx-1.5 rounded-lg transition-colors duration-100 data-[state=open]:bg-surface-control/30"
    >
      <div class="flex items-center gap-1 px-1 py-0.5">
        <AccordionTrigger as-child>
          <button
            type="button"
            class="group flex min-w-0 flex-1 items-center gap-2 rounded-md px-1 py-1 text-left transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
            :title="item.neighbor.data.title"
            :aria-label="`${t('spaces.knowledge.relations.inspect')}: ${item.neighbor.data.title}`"
          >
            <span
              class="grid size-7 shrink-0 place-items-center overflow-hidden rounded-[5px] bg-surface-control text-fg-tertiary ring-1 ring-stroke-divider/60 ring-inset"
            >
              <img
                v-if="item.thumbnail"
                :src="item.thumbnail"
                alt=""
                class="size-full object-cover"
                loading="lazy"
                decoding="async"
              />
              <component
                :is="nodeIcon(item.neighbor.data.kind)"
                v-else
                class="size-3.5"
                :stroke-width="1.5"
              />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block truncate text-caption font-medium text-fg-primary">{{
                item.neighbor.data.title
              }}</span>
              <span class="inline-flex items-center gap-1 text-caption-sm text-fg-tertiary">
                {{ t(`spaces.nodeTypes.${camelCase(item.neighbor.data.kind)}`) }}
                <ChevronDown
                  class="size-3 transition-transform duration-150 ease-standard group-data-[state=open]:rotate-180"
                  :stroke-width="1.6"
                />
              </span>
            </span>
          </button>
        </AccordionTrigger>
        <button
          type="button"
          class="mr-0.5 inline-flex shrink-0 items-center gap-1 rounded-md bg-surface-control/75 px-2 py-1 text-caption-sm text-fg-secondary transition-colors duration-100 hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :aria-label="`${t('spaces.knowledge.relations.kind')}: ${t(`spaces.knowledge.relations.kinds.${item.relation.kind}`)}`"
          @click="emit('editRelation', item.relation.id)"
        >
          {{ t(`spaces.knowledge.relations.kinds.${item.relation.kind}`) }}
          <Pencil class="size-2.5 text-fg-tertiary" :stroke-width="1.6" />
        </button>
      </div>
      <AccordionContent class="px-2 pb-2">
        <div class="ml-3 border-l border-stroke-divider/70 pl-3">
          <SpaceRelationEvidenceEditor :relation="item.relation" />
          <div class="mt-2 flex items-center justify-between pt-1">
            <button
              type="button"
              class="-ml-1.5 inline-flex items-center gap-1.5 rounded-md px-1.5 py-1 text-caption-sm text-fg-secondary transition-colors duration-100 hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
              @click="emit('reveal', item.neighbor.id)"
            >
              <LocateFixed class="size-3" :stroke-width="1.6" />
              {{ t("spaces.canvas.revealInCanvas") }}
            </button>
            <button
              type="button"
              class="-mr-1.5 inline-flex items-center gap-1.5 rounded-md px-1.5 py-1 text-caption-sm text-fg-tertiary transition-colors duration-100 hover:bg-surface-danger-subtle hover:text-fg-danger focus-visible:outline-2 focus-visible:outline-stroke-focus"
              @click="emit('removeRelation', item.relation.id, item.neighbor.data.title)"
            >
              <Trash2 class="size-3" :stroke-width="1.6" />
              {{ t("spaces.knowledge.relations.remove.action") }}
            </button>
          </div>
        </div>
      </AccordionContent>
    </AccordionItem>
  </div>
</template>
