<script setup lang="ts">
import type { Virtualizer } from "@tanstack/vue-virtual";
import { union, xor } from "es-toolkit";
import { TreeRoot, TreeVirtualizer } from "reka-ui";
import { computed, inject, nextTick, onBeforeUnmount, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
import { getCanvasEditorSession } from "../../canvas/editor-session";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { openSpaceNode } from "../../canvas/selection-actions";
import { createStructureIndex } from "../../model/structure/structure-index";
import { groupScopePathFor } from "../../model/structure/structure-queries";
import { useNodeOperationDrafts } from "../../session/node-operation-drafts";
import { useSpaceStore } from "../../session/space-store";
import { buildOutlineTreeItems, type OutlineTreeItem } from "./outline-tree-items";
import SpaceStructureTreeItem from "./SpaceStructureTreeItem.vue";
import { useStructureOutlineMenu } from "./use-structure-outline-menu";

const store = useSpaceStore();
const editor = getCanvasEditorSession();
const operations = useNodeOperationGate();
const drafts = useNodeOperationDrafts();
let renameLease: number | undefined;
const sidebar = inject(workspaceSidebarKey)!;
const { t } = useI18n();
const expandedIds = shallowRef<string[]>([]);
const editingId = shallowRef<string>();
const structure = computed(() => store.editSnapshot.structure);
const index = computed(() => createStructureIndex(structure.value));
const virtualizer = shallowRef<Virtualizer<Element | Window, Element>>();
const outlineRows = shallowRef<{ value: OutlineTreeItem }[]>([]);
const expandedSet = computed(() => new Set(expandedIds.value));
const selectedIds = computed(
  () => new Set([...editor.selection.value.structureNodeIds, ...editor.selection.value.nodeIds]),
);
const nodeById = computed(() => new Map(store.nodes.map((node) => [node.id, node])));
const items = computed(() =>
  buildOutlineTreeItems(index.value, new Map(store.nodes.map((node) => [node.id, node.data.title]))),
);

function outlineKey(item: OutlineTreeItem) {
  return item.id;
}

function toggle(id: string) {
  expandedIds.value = xor(expandedIds.value, [id]);
}

function expand(ids: readonly string[]) {
  expandedIds.value = union(expandedIds.value, ids);
}

function isSelected(id: string) {
  if (selectedIds.value.has(id)) return true;
  const structureNode = index.value.structureById.get(id);
  return structureNode?.kind === "content" && selectedIds.value.has(structureNode.nodeId);
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

async function rename(id: string) {
  if (!operations.allows("edit")) return;
  finishRename();
  renameLease = drafts.begin(id, "outline-title");
  if (!renameLease) return;
  const lease = renameLease;
  sidebar.reveal();
  expand(groupScopePathFor(id, index.value));
  await nextTick();
  if (renameLease !== lease) return;
  const row = outlineRows.value.findIndex(({ value }) => value.id === id);
  if (row >= 0) virtualizer.value?.scrollToIndex(row, { align: "auto" });
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
      <section class="relative h-full overflow-auto" :aria-label="t('spaces.sidebar.outline.title')">
        <TreeVirtualizer
          v-if="flattenItems.length"
          :estimate-size="30"
          :text-content="(item) => item.title"
          v-slot="{ item: { level, value }, virtualizer: itemVirtualizer, virtualItem }"
        >
          <SpaceStructureTreeItem
            :ref="
              (element) => {
                if (element) {
                  virtualizer = itemVirtualizer;
                  outlineRows = flattenItems as { value: OutlineTreeItem }[];
                }
              }
            "
            :item="value as OutlineTreeItem"
            :level="level"
            :selected="isSelected(value.id)"
            :expanded="expandedSet.has(value.id)"
            :editing="editingId === value.id"
            :save-name="(title) => saveName(value.id, title)"
            @select="select(value.id)"
            @open="open(value.id)"
            @menu="outlineMenu.show(value.id)"
            @toggle="toggle(value.id)"
            @finish="finishRename"
            @height="itemVirtualizer.resizeItem(virtualItem.index, $event)"
          />
        </TreeVirtualizer>
        <p v-if="flattenItems.length === 0" class="px-1 text-caption text-fg-tertiary">
          {{ t("spaces.sidebar.outline.empty") }}
        </p>
      </section>
    </template>
  </TreeRoot>
</template>
