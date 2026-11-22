<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import { useNodeDraft } from "../../inspection/presentation/use-node-draft";
import type { SpaceKnowledgeObjectPayload } from "../../model/nodes/node-payload";
import SpaceKnowledgeObjectEditor from "./SpaceKnowledgeObjectEditor.vue";
import SpaceKnowledgeObjectOverview from "./SpaceKnowledgeObjectOverview.vue";
import SpaceProvenanceSources from "./SpaceProvenanceSources.vue";

const props = defineProps<{ nodeId: string; data: SpaceKnowledgeObjectPayload }>();
const editing = shallowRef(false);
const draft = useNodeDraft(computed(() => props.nodeId));

watch(
  () => props.nodeId,
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
    <SpaceKnowledgeObjectEditor
      v-else
      :node-id="nodeId"
      :data="data"
      @cancel="finishEdit"
      @saved="finishEdit"
    />
    <SpaceProvenanceSources v-if="!editing" class="mt-3" target-kind="node" :target-id="nodeId" removable />
  </section>
</template>
