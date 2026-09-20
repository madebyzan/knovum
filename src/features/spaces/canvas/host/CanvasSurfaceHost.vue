<script setup lang="ts">
import { isEqual } from "es-toolkit";
import { onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";

import type { CanvasExchangePort } from "../../canvas-exchange/canvas-exchange-port";
import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportDraft,
  CanvasJsonImportRequest,
} from "../../canvas-exchange/json-canvas-import-contract";
import { jsonCanvasCodec } from "../../canvas-exchange/json-canvas-repository";
import { createJsonCanvasExchangePort } from "../../canvas-exchange/json-canvas-service";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { CanvasHostEvents, CanvasSurfaceEvents } from "../contract/canvas-editor";
import type { CanvasNodeContentSlotProps } from "../contract/canvas-surface";
import type { CanvasTool } from "../contract/canvas-tool";
import KonvaCanvasSurface from "../konva/KonvaCanvasSurface.vue";
import { rebaseCanvasDocument } from "./canvas-document-rebase";
import { useCanvasDocumentSession } from "./use-canvas-document-session";

const props = defineProps<{
  events: CanvasHostEvents;
  importJsonCanvas(request: CanvasJsonImportRequest): Promise<CanvasJsonImportCommitResult>;
  focusNodeIds: readonly string[];
  interactionEnabled: boolean;
  tool: CanvasTool;
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
  spaceId: string;
}>();
const emit = defineEmits<{ exchangeReady: [exchange: CanvasExchangePort] }>();
defineSlots<{ "node-content"(props: CanvasNodeContentSlotProps): unknown }>();
const session = useCanvasDocumentSession({
  nodes: () => props.nodes,
  reportError: props.events.onError,
  spaceId: () => props.spaceId,
});
const surface = useTemplateRef<InstanceType<typeof KonvaCanvasSurface>>("surface");

const surfaceEvents: CanvasSurfaceEvents = {
  ...props.events,
  onExecuteSpaceEdit(request) {
    return session.commitTransaction((baseRevision) => {
      const current = session.document.value!;
      if (
        request.command.kind !== "measure-text" &&
        (!isEqual(request.baseDocument.structure, current.structure) ||
          !isEqual(request.baseDocument.layout, current.layout) ||
          !isEqual(request.baseDocument.relationStyles, current.relationStyles))
      )
        throw new Error("The Space changed during this edit. Please retry the operation.");
      return props.events.onExecuteSpaceEdit({
        mutationId: request.mutationId,
        command: request.command,
        baseRevision,
      });
    });
  },
  onReplaySpaceEdit(request) {
    return session.commitTransaction((baseRevision) =>
      props.events.onReplaySpaceEdit({ ...request, baseRevision }),
    );
  },
  onViewportChange(viewport) {
    if (session.document.value) session.documentChanged({ ...session.document.value, viewport });
    props.events.onViewportChange(viewport);
  },
  async onCommitGesture(request) {
    return session.commitTransaction((baseRevision) =>
      props.events.onCommitGesture({
        ...request,
        baseRevision,
        document: rebaseCanvasDocument(request.baseDocument, session.document.value!, request.document),
      }),
    );
  },
  async onCutSelection(request) {
    return session.commitTransaction((baseRevision) =>
      props.events.onCutSelection({ ...request, baseRevision }),
    );
  },
  async onDeleteSelection(request) {
    return session.commitTransaction((baseRevision) =>
      props.events.onDeleteSelection({ ...request, baseRevision }),
    );
  },
  onHistoryTransition(nextDocument, transition, baseDocument) {
    return session.commitTransaction(async (baseRevision) => ({
      ...(await transition({
        baseRevision,
        document: rebaseCanvasDocument(baseDocument, session.document.value!, nextDocument),
      })),
      document: rebaseCanvasDocument(baseDocument, session.document.value!, nextDocument),
    }));
  },
  onReady(editor) {
    props.events.onReady(editor);
    emit(
      "exchangeReady",
      createJsonCanvasExchangePort({
        codec: jsonCanvasCodec,
        editor,
        nodes: () => props.nodes,
        relations: () => props.relations,
        commitImport: commitJsonCanvasImport,
        historyEntry: props.events.onHistoryEntry,
        historyTransition: surfaceEvents.onHistoryTransition,
        historyTransaction: props.events.onHistoryTransaction,
      }),
    );
  },
};

function commitJsonCanvasImport(request: CanvasJsonImportDraft) {
  return session.commitTransaction((baseRevision) => props.importJsonCanvas({ ...request, baseRevision }));
}

watch(
  () => props.spaceId,
  () => {
    surface.value?.flushViewport();
    void session.start().catch(props.events.onError);
  },
);
watch(
  () => props.nodes,
  (nodes) => {
    session.synchronizeNodes(nodes);
  },
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
    :events="surfaceEvents"
    :focus-node-ids="focusNodeIds"
    :interaction-enabled="interactionEnabled"
    :tool="tool"
    :nodes="nodes"
    :relations="relations"
    @document-change="session.documentChanged"
  >
    <template #node-content="content">
      <slot name="node-content" v-bind="content" />
    </template>
  </KonvaCanvasSurface>
</template>
