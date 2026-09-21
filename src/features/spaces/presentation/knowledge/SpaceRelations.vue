<script setup lang="ts">
import { ArrowDownLeft, ArrowUpRight, Link2 } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
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
const expandedId = ref<string>();

const selected = computed(() => {
  const node = selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined;
  return node && isSpaceSemanticNode(node) ? node : undefined;
});

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
    const neighbor = nodes.value.find((node) => node.id === neighborId);
    return neighbor ? [{ relation, neighbor, isOutgoing }] : [];
  });
}

function reveal(id: string) {
  selectAndRevealSpaceNode(id);
}

function toggleExpand(id: string) {
  expandedId.value = expandedId.value === id ? undefined : id;
}

function editRelation(id: string) {
  const neighbor = [...incoming.value, ...outgoing.value].find((candidate) => candidate.relation.id === id);
  if (!neighbor) return;
  void showSpaceRelationMenu(
    neighbor.relation.kind,
    async (kind) => {
      await store.updateRelationKind(id, kind);
    },
    () => store.removeRelation(id),
  );
}
</script>

<template>
  <section v-if="selected">
    <div
      v-if="incoming.length === 0 && outgoing.length === 0"
      class="flex items-start gap-2 text-caption text-foreground-tertiary"
    >
      <Link2 class="mt-0.5 size-(--knovum-icon-size-xs) shrink-0" />
      <span>{{ t("spaces.knowledge.relationsEmpty") }}</span>
    </div>

    <template v-else>
      <SpaceRelationGroup
        v-if="incoming.length"
        :relations="incoming"
        :icon="ArrowDownLeft"
        label-key="spaces.knowledge.incomingRelations"
        direction-key="spaces.canvas.actions.incoming"
        :expanded-id="expandedId"
        @reveal="reveal"
        @toggle-expand="toggleExpand"
        @edit-relation="editRelation"
      />

      <SpaceRelationGroup
        v-if="outgoing.length"
        :class="incoming.length && 'mt-4'"
        :relations="outgoing"
        :icon="ArrowUpRight"
        label-key="spaces.knowledge.outgoingRelations"
        direction-key="spaces.canvas.actions.outgoing"
        :expanded-id="expandedId"
        @reveal="reveal"
        @toggle-expand="toggleExpand"
        @edit-relation="editRelation"
      />
    </template>
  </section>
</template>
