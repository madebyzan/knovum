<script setup lang="ts">
import { uniq } from "es-toolkit";
import { TreeRoot } from "reka-ui";
import { computed, inject, onBeforeUnmount, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../shared/contracts/workspace-sidebar";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { openSpaceNode } from "../canvas/selection-actions";
import { groupScopePathFor } from "../model/structure/structure-queries";
import { validateStructureSnapshot } from "../model/structure/structure-validation";
import { useNodeOperationDrafts } from "../session/node-operation-drafts";
import { useSpaceStore } from "../session/space-store";
import { buildOutlineTreeItems, type OutlineTreeItem } from "./outline-tree-items";
import SpaceStructureOutlineRow from "./SpaceStructureOutlineRow.vue";
import { useStructureOutlineMenu } from "./use-structure-outline-menu";

const store = useSpaceStore();
const editor = useCanvasEditorSession();
const operations = useNodeOperationGate();
const drafts = useNodeOperationDrafts();
let renameLease: number | undefined;
const sidebar = inject(workspaceSidebarKey)!;
const { t } = useI18n();
const expandedIds = shallowRef<string[]>([]);
const editingId = shallowRef<string>();
const index = computed(() =>
  validateStructureSnapshot(store.editSnapshot.structure, store.editSnapshot.layouts),
);
const nodeById = computed(() => new Map(store.nodes.map((node) => [node.id, node])));
const items = computed(() =>
  buildOutlineTreeItems(index.value, new Map(store.nodes.map((node) => [node.id, node.data.title]))),
);

function outlineKey(item: OutlineTreeItem) {
  return item.id;
}

function toggle(id: string) {
  expandedIds.value = expandedIds.value.includes(id)
    ? expandedIds.value.filter((value) => value !== id)
    : [...expandedIds.value, id];
}

function expand(ids: readonly string[]) {
  expandedIds.value = uniq([...expandedIds.value, ...ids]);
}

function isSelected(id: string) {
  if (editor.selection.value.structureNodeIds.includes(id)) return true;
  const structureNode = index.value.structureById.get(id);
  return structureNode?.kind === "content" && editor.selection.value.nodeIds.includes(structureNode.nodeId);
}

function select(id: string) {
  const structureNode = index.value.structureById.get(id);
  if (structureNode?.kind === "content") {
    editor.selectNodes([structureNode.nodeId]);
    editor.reveal([structureNode.nodeId], "ensure-visible");
  } else if (structureNode) {
    editor.selectStructureNodes([structureNode.id]);
    editor.reveal([structureNode.id], "ensure-visible");
  }
}

async function open(id: string) {
  const structureNode = index.value.structureById.get(id);
  if (!structureNode) return;
  if (structureNode.kind === "group") {
    store.groupScopePath = [...groupScopePathFor(structureNode.id, index.value), structureNode.id];
    expand([structureNode.id]);
    editor.selectStructureNodes([structureNode.id]);
    return;
  }
  if (structureNode.kind === "section") {
    editor.reveal([structureNode.id], "focus");
    return;
  }
  const node = nodeById.value.get(structureNode.nodeId);
  if (
    node &&
    operations.allows(node.data.kind === "note" || node.data.kind === "markdown" ? "edit" : "inspect")
  )
    await openSpaceNode(store, node);
}

function rename(id: string) {
  if (!operations.allows("edit")) return;
  finishRename();
  renameLease = drafts.begin(id, "outline-title");
  if (!renameLease) return;
  sidebar.reveal();
  expand(groupScopePathFor(id, index.value));
  editingId.value = id;
}

function finishRename() {
  if (renameLease !== undefined) drafts.end(renameLease);
  renameLease = undefined;
  editingId.value = undefined;
}

async function saveName(id: string, title: string) {
  if (editingId.value !== id || renameLease === undefined) return;
  const structureNode = index.value.structureById.get(id);
  if (!structureNode) throw new Error("Structure node no longer exists");
  if (structureNode.kind === "content") return store.renameNode(structureNode.nodeId, title);
  return store.executeSpaceEdit({ kind: "container.rename", structureNodeId: id, title });
}

const outlineMenu = useStructureOutlineMenu({
  index,
  rename,
  reveal: select,
  snapshot: () => store.editSnapshot,
  store,
});

watch(
  () => store.outlineRenameStructureNodeId,
  (structureNodeId) => {
    if (!structureNodeId) return;
    store.outlineRenameStructureNodeId = null;
    if (index.value.structureById.has(structureNodeId)) rename(structureNodeId);
  },
  { immediate: true },
);
watch(
  () => store.activeSpaceId,
  () => {
    finishRename();
    expandedIds.value = [];
  },
);
onBeforeUnmount(finishRename);
</script>

<template>
  <TreeRoot as-child :items="items" :get-key="outlineKey" v-model:expanded="expandedIds">
    <template #default="{ flattenItems }">
      <section class="mt-2 space-y-0.5" :aria-label="t('spaces.sidebar.outline.title')">
        <SpaceStructureOutlineRow
          v-for="{ level, value } in flattenItems"
          :key="value.id"
          :item="value"
          :level="level"
          :selected="isSelected(value.id)"
          :expanded="expandedIds.includes(value.id)"
          :editing="editingId === value.id"
          :save-name="(title) => saveName(value.id, title)"
          @select="select(value.id)"
          @open="open(value.id)"
          @menu="outlineMenu.show(value.id)"
          @toggle="toggle(value.id)"
          @finish="finishRename"
        />
        <p v-if="flattenItems.length === 0" class="px-1 text-caption text-foreground-tertiary">
          {{ t("spaces.sidebar.outline.empty") }}
        </p>
      </section>
    </template>
  </TreeRoot>
</template>
