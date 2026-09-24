<script setup lang="ts">
import { Sparkles, TriangleAlert } from "@lucide/vue";
import { computed, shallowRef, toRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { refreshSpaceAsset } from "../assets/asset-operations";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { useDerivationMetadata } from "../knowledge-derivations/use-derivation-metadata";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import { createStructureIndex } from "../model/structure/structure-index";
import { useSpaceStore } from "../session/space-store";
import SpaceGroundedDocumentDetails from "./knowledge/SpaceGroundedDocumentDetails.vue";
import SpaceDerivationFailureDetails from "./SpaceDerivationFailureDetails.vue";
import SpaceDerivationMetadataView from "./SpaceDerivationMetadataView.vue";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const store = useSpaceStore();
const operations = useNodeOperationGate();
const nodeRef = toRef(props, "node");
const { documentMetadata, imageMetadata, videoMetadata, failure } = useDerivationMetadata(nodeRef);

const retrying = shallowRef(false);
const canRetry = computed(() => {
  if (!props.node) return false;
  return isSpaceManagedAssetPayload(props.node.data) || props.node.data.kind === "markdown";
});

async function onRetry() {
  const spaceId = store.activeSpaceId;
  if (!spaceId || retrying.value || !operations.allows("edit")) return;
  retrying.value = true;
  try {
    if (isSpaceManagedAssetPayload(props.node.data)) {
      const refreshed = await refreshSpaceAsset(spaceId, props.node.id);
      if (refreshed) store.applyNodeDataForSpace(spaceId, props.node.id, refreshed);
    } else if (props.node.data.kind === "markdown") {
      store.applyNodeDataForSpace(spaceId, props.node.id, {
        ...props.node.data,
        derivationState: undefined,
      });
      await store.saveNodeDataForSpace(spaceId, props.node.id, props.node.data, {
        ...props.node.data,
        derivationState: undefined,
      });
    }
    toast.success(t("common.retry"));
  } catch (error) {
    toast.error(String(error));
  } finally {
    retrying.value = false;
  }
}

async function removeFailedOutput() {
  if (!operations.allows("structure")) return;
  const structureNodeId = createStructureIndex(store.editSnapshot.structure).nodeIdToStructureNodeId.get(
    props.node.id,
  );
  if (!structureNodeId) throw new Error("Derived output is unavailable in the Space structure");
  await store.executeSpaceEdit({ kind: "structure.delete-subtrees", rootIds: [structureNodeId] });
}
</script>

<template>
  <SpaceInspectorSection
    v-if="failure"
    :title="$t('spaces.creation.failedAttempt')"
    :icon="TriangleAlert"
    :default-expanded="true"
  >
    <SpaceDerivationFailureDetails
      :failure="failure"
      :can-retry="canRetry && operations.allows('edit')"
      :retrying="retrying"
      @retry="onRetry"
      @remove="removeFailedOutput"
    />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="imageMetadata"
    :title="$t('spaces.creation.knowledgeDerivation')"
    :icon="Sparkles"
    :default-expanded="true"
  >
    <SpaceDerivationMetadataView :metadata="imageMetadata" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="videoMetadata"
    :title="$t('spaces.creation.knowledgeDerivation')"
    :icon="Sparkles"
    :default-expanded="true"
  >
    <SpaceDerivationMetadataView :metadata="videoMetadata" />
  </SpaceInspectorSection>

  <SpaceInspectorSection
    v-if="documentMetadata"
    :title="$t('spaces.creation.knowledgeDerivation')"
    :icon="Sparkles"
    :default-expanded="true"
  >
    <SpaceGroundedDocumentDetails :metadata="documentMetadata" />
  </SpaceInspectorSection>
</template>
