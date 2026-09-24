<script setup lang="ts">
import { computed, inject, onBeforeUnmount, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../shared/contracts/workspace-sidebar";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { openSpaceNode } from "../canvas/selection-actions";
import { projectStructureOutline } from "../model/structure/outline-projection";
import { groupScopePathFor } from "../model/structure/structure-queries";
import { validateStructureSnapshot } from "../model/structure/structure-validation";
import { useNodeOperationDrafts } from "../session/node-operation-drafts";
import { useSpaceStore } from "../session/space-store";
import SpaceStructureOutlineRow from "./SpaceStructureOutlineRow.vue";
import { useOutlineTreeNavigation } from "./use-outline-tree-navigation";
import { useStructureOutlineMenu } from "./use-structure-outline-menu";

const store = useSpaceStore();
const editor = useCanvasEditorSession();
const operations = useNodeOperationGate();
const drafts = useNodeOperationDrafts();
let renameLease: number | undefined;
const sidebar = inject(workspaceSidebarKey)!;
const { t } = useI18n();
const expandedIds = shallowRef(new Set<string>());
const editingId = shallowRef<string>();
const tree = shallowRef<HTMLElement | null>(null);
const index = computed(() =>
  validateStructureSnapshot(store.editSnapshot.structure, store.editSnapshot.layouts),
);
const nodeById = computed(() => new Map(store.nodes.map((node) => [node.id, node])));
const rows = computed(() =>
  projectStructureOutline(
    index.value,
    new Map(store.nodes.map((node) => [node.id, node.data.title])),
    expandedIds.value,
  ),
);

const treeNavigation = useOutlineTreeNavigation({ expandedIds, rows, root: tree, toggle });

function toggle(id: string) {
  const next = new Set(expandedIds.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  expandedIds.value = next;
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
    expandedIds.value = new Set([...expandedIds.value, structureNode.id]);
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
  for (const ancestor of groupScopePathFor(id, index.value)) {
    expandedIds.value = new Set([...expandedIds.value, ancestor]);
  }
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
    expandedIds.value = new Set();
  },
);
onBeforeUnmount(finishRename);
</script>

<template>
  <section ref="tree" class="mt-2 space-y-0.5" role="tree" :aria-label="t('spaces.navigation.outline')">
    <SpaceStructureOutlineRow
      v-for="(row, index) in rows"
      :key="row.id"
      :row="row"
      :tab-index="treeNavigation.tabIndex(row.id, index)"
      :selected="isSelected(row.id)"
      :expanded="expandedIds.has(row.id)"
      :editing="editingId === row.id"
      :save-name="(title) => saveName(row.id, title)"
      @select="select(row.id)"
      @open="open(row.id)"
      @menu="outlineMenu.show(row.id)"
      @toggle="toggle(row.id)"
      @finish="finishRename"
      @focus="treeNavigation.setFocusedId(row.id)"
      @navigate="treeNavigation.handleKeydown($event, row, index)"
    />
    <p v-if="rows.length === 0" class="px-1 text-caption text-foreground-tertiary">
      {{ t("spaces.navigation.outlineEmpty") }}
    </p>
  </section>
</template>
