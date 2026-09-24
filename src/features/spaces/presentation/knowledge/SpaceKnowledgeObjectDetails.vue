<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import { SYSTEM_OBJECT_TYPE_IDS } from "../../knowledge/model/object-types";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceKnowledgeObjectPayload } from "../../model/nodes/node-payload";
import { useNodeDraft } from "../use-node-draft";
import SpaceKnowledgeObjectEditor from "./SpaceKnowledgeObjectEditor.vue";
import SpaceKnowledgeObjectOverview from "./SpaceKnowledgeObjectOverview.vue";
import SpaceProvenanceSources from "./SpaceProvenanceSources.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const editing = shallowRef(false);
const draft = useNodeDraft(
  computed(() => props.node.id),
  "knowledge-object",
);
const data = computed(() => props.node.data as SpaceKnowledgeObjectPayload);
const isEvidence = computed(() => data.value.objectTypeId === SYSTEM_OBJECT_TYPE_IDS.evidence);

watch(
  () => props.node.id,
  () => (editing.value = false),
);
function beginEdit() {
  if (draft.begin()) editing.value = true;
}
function finishEdit() {
  editing.value = false;
  draft.end();
}
</script>

<template>
  <section>
    <SpaceKnowledgeObjectOverview
      v-if="!editing"
      :data="data"
      :edit-disabled="!draft.available.value"
      @edit="beginEdit"
    />
    <SpaceKnowledgeObjectEditor v-else :node="node" @cancel="finishEdit" @saved="finishEdit" />
    <SpaceProvenanceSources
      v-if="!editing && isEvidence"
      class="mt-3"
      target-kind="node"
      :target-id="node.id"
      removable
    />
  </section>
</template>
