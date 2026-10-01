<script setup lang="ts">
import { ArrowDownLeft, ArrowUpRight, Link2 } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { AccordionRoot } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import { spaceCanvasOverviewSource } from "../../assets/node-preview-source";
import { useCanvasSelectionProjection } from "../../canvas/editor-session";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import { isSymmetricRelationKind } from "../../model/relations/relation";
import { useSpaceStore } from "../../session/space-store";
import { showSpaceRelationKindMenu } from "./relation-kind-menu";
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
const symmetric = computed(() => groupedRelations("symmetric"));
const incoming = computed(() => groupedRelations("incoming"));
const outgoing = computed(() => groupedRelations("outgoing"));

function groupedRelations(direction: "symmetric" | "incoming" | "outgoing") {
  const id = selected.value?.id;
  if (!id) return [];
  return relationState.value.flatMap((relation) => {
    const matchesOutgoing = relation.sourceId === id;
    const matchesIncoming = relation.targetId === id;
    const isSymmetric = isSymmetricRelationKind(relation.kind);
    const matches =
      direction === "symmetric"
        ? isSymmetric && (matchesOutgoing || matchesIncoming)
        : !isSymmetric && (direction === "outgoing" ? matchesOutgoing : matchesIncoming);
    if (!matches) return [];
    const neighborId = matchesOutgoing ? relation.targetId : relation.sourceId;
    const neighbor = nodeById.value.get(neighborId);
    return neighbor ? [{ relation, neighbor, thumbnail: spaceCanvasOverviewSource(neighbor.data) }] : [];
  });
}

function reveal(id: string) {
  selectAndRevealSpaceNode(id);
}

function editRelation(id: string) {
  const neighbor = [...symmetric.value, ...incoming.value, ...outgoing.value].find(
    (candidate) => candidate.relation.id === id,
  );
  if (!neighbor) return;
  void showSpaceRelationKindMenu(neighbor.relation.kind, async (kind) => {
    await store.updateRelationKind(id, kind);
  });
}

async function removeRelation(id: string, neighborTitle: string) {
  const confirmed = await confirmAction({
    title: t("spaces.knowledge.relations.remove"),
    description: t("spaces.knowledge.relations.removeConfirm", { title: neighborTitle }),
    confirmLabel: t("spaces.knowledge.relations.remove"),
    danger: true,
  });
  if (!confirmed) return;
  try {
    await store.revokeRelation(id);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <section v-if="selected">
    <div
      v-if="symmetric.length === 0 && incoming.length === 0 && outgoing.length === 0"
      class="flex items-start gap-2 text-caption text-fg-tertiary"
    >
      <Link2 class="mt-0.5 size-3 shrink-0" />
      <span>{{ t("spaces.knowledge.relations.empty") }}</span>
    </div>

    <AccordionRoot v-else type="single" collapsible>
      <SpaceRelationGroup
        v-if="symmetric.length"
        :relations="symmetric"
        :icon="Link2"
        label-key="spaces.knowledge.relations.symmetric"
        @reveal="reveal"
        @edit-relation="editRelation"
        @remove-relation="removeRelation"
      />

      <SpaceRelationGroup
        v-if="incoming.length"
        :class="symmetric.length && 'mt-3'"
        :relations="incoming"
        :icon="ArrowDownLeft"
        label-key="spaces.knowledge.relations.incoming"
        @reveal="reveal"
        @edit-relation="editRelation"
        @remove-relation="removeRelation"
      />

      <SpaceRelationGroup
        v-if="outgoing.length"
        :class="(symmetric.length || incoming.length) && 'mt-3'"
        :relations="outgoing"
        :icon="ArrowUpRight"
        label-key="spaces.knowledge.relations.outgoing"
        @reveal="reveal"
        @edit-relation="editRelation"
        @remove-relation="removeRelation"
      />
    </AccordionRoot>
  </section>
</template>
