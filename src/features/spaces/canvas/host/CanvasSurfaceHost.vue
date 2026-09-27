<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";

import type { CanvasExchangePort } from "../../canvas-exchange/canvas-exchange-port";
import { createJsonCanvasExchangePort } from "../../canvas-exchange/json-canvas-exchange";
import type {
  CanvasImportCommit,
  CanvasJsonImportDraft,
  CanvasJsonImportRequest,
} from "../../canvas-exchange/json-canvas-import-contract";
import type { SpaceRelation } from "../../model/relations/relation";
import type { CanvasHostEvents, CanvasSurfaceEvents } from "../contract/canvas-editor";
import type { CanvasNodeContentSlotProps, CanvasSceneInput } from "../contract/canvas-surface";
import type { CanvasTool } from "../contract/canvas-tool";
import type { SpacePersistenceState } from "../contract/space-persistence-state";
import KonvaCanvasSurface from "../konva/KonvaCanvasSurface.vue";
import { useCanvasDocumentSession } from "./use-canvas-document-session";

const props = defineProps<{
  events: CanvasHostEvents;
  persistence: SpacePersistenceState;
  ensurePersisted(): Promise<void>;
  importJsonCanvas(request: CanvasJsonImportRequest): Promise<CanvasImportCommit>;
  focusNodeIds: readonly string[];
  groupScopePath: readonly string[];
  interactionEnabled: boolean;
  tool: CanvasTool;
  sceneInput: CanvasSceneInput;
  exchangeRelations: readonly SpaceRelation[];
  spaceId: string;
}>();
const emit = defineEmits<{ exchangeReady: [exchange: CanvasExchangePort] }>();
defineSlots<{ "node-content"(props: CanvasNodeContentSlotProps): unknown }>();
const session = useCanvasDocumentSession({
  snapshot: () => props.sceneInput.editSnapshot,
  persistence: () => props.persistence,
  ensurePersisted: () => props.ensurePersisted(),
  reportError: props.events.onError,
  spaceId: () => props.spaceId,
});
const surface = useTemplateRef<InstanceType<typeof KonvaCanvasSurface>>("surface");

const surfaceEvents: CanvasSurfaceEvents = {
  ...props.events,
  onViewportChange(viewport) {
    session.viewportChanged(viewport);
    props.events.onViewportChange(viewport);
  },
  onReady(editor) {
    props.events.onReady(editor);
    emit(
      "exchangeReady",
      createJsonCanvasExchangePort({
        snapshot: () => props.sceneInput.editSnapshot,
        nodes: () => props.sceneInput.nodes,
        relations: () => props.exchangeRelations,
        commitImport: commitJsonCanvasImport,
        transaction: props.events.onHistoryTransaction,
      }),
    );
  },
};

function commitJsonCanvasImport(request: CanvasJsonImportDraft) {
  return props.importJsonCanvas({ ...request, baseRevision: props.sceneInput.editSnapshot.editRevision });
}

watch(
  () => props.spaceId,
  () => {
    surface.value?.flushViewport();
    void session.start().catch(props.events.onError);
  },
);
watch(
  () => props.persistence,
  (persistence) => {
    if (persistence === "persisted") void session.persist().catch(props.events.onError);
  },
);
watch(
  () => props.sceneInput.editSnapshot,
  () => session.synchronizeSnapshot(),
);
onMounted(() => void session.start().catch(props.events.onError));
onBeforeUnmount(() => {
  surface.value?.flushViewport();
  session.dispose();
});
</script>

<template>
  <KonvaCanvasSurface
    ref="surface"
    v-if="session.document.value"
    :key="session.surfaceKey.value"
    :document="session.document.value"
    :scene-input="sceneInput"
    :events="surfaceEvents"
    :focus-node-ids="focusNodeIds"
    :group-scope-path="groupScopePath"
    :interaction-enabled="interactionEnabled"
    :tool="tool"
    @document-change="session.documentChanged"
  >
    <template #node-content="content">
      <slot name="node-content" v-bind="content" />
    </template>
  </KonvaCanvasSurface>
</template>
