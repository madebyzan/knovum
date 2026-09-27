<script setup lang="ts">
import { ArrowDownLeft, ArrowUpRight, Link2 } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { AccordionRoot } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useCanvasSelectionProjection } from "../../canvas/editor-session";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import { showSpaceRelationMenu } from "../../knowledge/presentation/relation-menu";
import { useSpaceStore } from "../../session/space-store";
import SpaceRelationGroup from "./SpaceRelationGroup.vue";

const store = useSpaceStore();
const { nodes, relations: relationState } = storeToRefs(store);
const { selectedNodes } = useCanvasSelectionProjection(nodes);
const { t } = useI18n();

const selected = computed(() => {
  const node = selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined;
  return node && isSpaceSemanticNode(node) ? node : undefined;
});

const nodeById = computed(() => new Map(nodes.value.map((node) => [node.id, node])));
const incoming = computed(() => groupedRelations(false));
const outgoing = computed(() => groupedRelations(true));

function groupedRelations(isOutgoing: boolean) {
  const id = selected.value?.id;
  if (!id) return [];
  return relationState.value.flatMap((relation) => {
    const matchesOutgoing = relation.sourceId === id;
    const matchesIncoming = relation.targetId === id;
    if ((isOutgoing && !matchesOutgoing) || (!isOutgoing && !matchesIncoming)) return [];
    const neighborId = isOutgoing ? relation.targetId : relation.sourceId;
    const neighbor = nodeById.value.get(neighborId);
    return neighbor ? [{ relation, neighbor }] : [];
  });
}

function reveal(id: string) {
  selectAndRevealSpaceNode(id);
}

function editRelation(id: string) {
  const neighbor = [...incoming.value, ...outgoing.value].find((candidate) => candidate.relation.id === id);
  if (!neighbor) return;
  void showSpaceRelationMenu(
    neighbor.relation.kind,
    async (kind) => {
      await store.updateRelationKind(id, kind);
    },
    () => store.revokeRelation(id),
  );
}
</script>

<template>
  <section v-if="selected">
    <div
      v-if="incoming.length === 0 && outgoing.length === 0"
      class="flex items-start gap-2 text-caption text-fg-tertiary"
    >
      <Link2 class="mt-0.5 size-3 shrink-0" />
      <span>{{ t("spaces.knowledge.relations.empty") }}</span>
    </div>

    <AccordionRoot v-else type="single" collapsible>
      <SpaceRelationGroup
        v-if="incoming.length"
        :relations="incoming"
        :icon="ArrowDownLeft"
        label-key="spaces.knowledge.relations.incoming"
        direction-key="spaces.canvas.relations.incoming"
        @reveal="reveal"
        @edit-relation="editRelation"
      />

      <SpaceRelationGroup
        v-if="outgoing.length"
        :class="incoming.length && 'mt-4'"
        :relations="outgoing"
        :icon="ArrowUpRight"
        label-key="spaces.knowledge.relations.outgoing"
        direction-key="spaces.canvas.relations.outgoing"
        @reveal="reveal"
        @edit-relation="editRelation"
      />
    </AccordionRoot>
  </section>
</template>
