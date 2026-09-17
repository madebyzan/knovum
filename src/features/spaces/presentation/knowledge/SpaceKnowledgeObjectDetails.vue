<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import { SYSTEM_OBJECT_TYPE_IDS } from "../../knowledge/model/object-types";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceKnowledgeObjectPayload } from "../../model/nodes/node-payload";
import SpaceKnowledgeObjectEditor from "./SpaceKnowledgeObjectEditor.vue";
import SpaceKnowledgeObjectOverview from "./SpaceKnowledgeObjectOverview.vue";
import SpaceProvenanceSources from "./SpaceProvenanceSources.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const editing = shallowRef(false);
const data = computed(() => props.node.data as SpaceKnowledgeObjectPayload);
const isEvidence = computed(() => data.value.objectTypeId === SYSTEM_OBJECT_TYPE_IDS.evidence);

watch(
  () => props.node.id,
  () => (editing.value = false),
);
</script>

<template>
  <section>
    <SpaceKnowledgeObjectOverview v-if="!editing" :data="data" @edit="editing = true" />
    <SpaceKnowledgeObjectEditor v-else :node="node" @cancel="editing = false" @saved="editing = false" />
    <SpaceProvenanceSources
      v-if="!editing && isEvidence"
      class="mt-3"
      target-kind="node"
      :target-id="node.id"
      removable
    />
  </section>
</template>
