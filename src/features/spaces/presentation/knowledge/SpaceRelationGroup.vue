<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import type { Component } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import SpaceRelationDetails from "./SpaceRelationDetails.vue";

export interface SpaceRelationNeighbor {
  relation: SpaceRelation;
  neighbor: SpaceCanvasNode;
}

defineProps<{
  relations: SpaceRelationNeighbor[];
  icon: Component;
  labelKey: string;
  directionKey: string;
  expandedId?: string | undefined;
}>();

const emit = defineEmits<{
  reveal: [neighborId: string];
  toggleExpand: [relationId: string];
  editRelation: [relationId: string];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="space-y-2">
    <h4 class="flex items-center gap-1.5 text-micro font-medium text-foreground-tertiary">
      <component :is="icon" class="size-(--knovum-icon-size-xs)" />
      {{ t(labelKey, { count: relations.length }) }}
    </h4>
    <template v-for="neighbor in relations" :key="neighbor.relation.id">
      <div
        class="group rounded-control border border-transparent bg-control/30 p-2 hover:border-divider hover:bg-control/50"
      >
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="min-w-0 flex-1 text-left text-caption font-medium text-foreground"
            @click="emit('reveal', neighbor.neighbor.id)"
          >
            {{ neighbor.neighbor.data.title }}
          </button>
          <UiIconButton
            :label="t('spaces.knowledge.relationDetails')"
            size="sm"
            :aria-expanded="expandedId === neighbor.relation.id"
            @click="emit('toggleExpand', neighbor.relation.id)"
          >
            <ChevronDown
              class="size-(--knovum-icon-size-xs) transition-transform duration-150 ease-standard"
              :class="expandedId === neighbor.relation.id ? 'rotate-180' : ''"
            />
          </UiIconButton>
        </div>
        <div class="mt-1 flex items-center gap-2">
          <span class="text-micro text-foreground-tertiary">{{ t(directionKey) }}</span>
          <button
            type="button"
            class="rounded-control bg-control px-1.5 py-0.5 text-micro text-foreground-secondary hover:bg-control-hover hover:text-foreground"
            @click="emit('editRelation', neighbor.relation.id)"
          >
            {{ t(`spaces.knowledge.relation.${neighbor.relation.kind}`) }}
          </button>
        </div>
      </div>
      <SpaceRelationDetails v-if="expandedId === neighbor.relation.id" :relation="neighbor.relation" />
    </template>
  </div>
</template>
