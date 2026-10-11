<script setup lang="ts">
import type { Virtualizer } from "@tanstack/vue-virtual";
import { union, xor } from "es-toolkit";
import { TreeRoot, TreeVirtualizer } from "reka-ui";
import { computed, inject, nextTick, onBeforeUnmount, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
import { getCanvasEditorSession } from "../../canvas/editor-session";
import { openSpaceNode } from "../../canvas/selection-actions";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { ancestorsOf, createStructureIndex } from "../../model/structure/structure-index";
import { groupScopePathFor } from "../../model/structure/structure-queries";
import { useNodeOperationDraftsStore } from "../../session/node-operation-drafts-store";
import { useSpaceStore } from "../../session/space-store";
import { buildOutlineTreeItems, type OutlineTreeItem } from "./outline-tree-items";
import SpaceStructureTreeItem from "./SpaceStructureTreeItem.vue";
import { useStructureOutlineMenu } from "./use-structure-outline-menu";

const store = useSpaceStore();
const editor = getCanvasEditorSession();
const operations = useNodeOperationGate();
const drafts = useNodeOperationDraftsStore();
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

function toggle(structureNodeId: string) {
  expandedIds.value = xor(expandedIds.value, [structureNodeId]);
}

function expand(structureNodeIds: readonly string[]) {
  expandedIds.value = union(expandedIds.value, structureNodeIds);
}

function isSelected(structureNodeId: string) {
  if (selectedIds.value.has(structureNodeId)) return true;
  const structureNode = index.value.structureById.get(structureNodeId);
  return structureNode?.kind === "content" && selectedIds.value.has(structureNode.nodeId);
}

function select(structureNodeId: string) {
  const structureNode = index.value.structureById.get(structureNodeId);
  if (structureNode?.kind === "content") {
    editor.selectNodes([structureNode.nodeId]);
    editor.reveal([structureNode.nodeId], "ensure-visible");
  } else if (structureNode) {
    editor.selectStructureNodes([structureNode.id]);
    editor.reveal([structureNode.id], "ensure-visible");
  }
}

async function open(structureNodeId: string) {
  const structureNode = index.value.structureById.get(structureNodeId);
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
    await openSpaceNode(node);
}

async function rename(structureNodeId: string) {
  if (!operations.allows("edit")) return;
  finishRename();
  renameLease = drafts.begin(structureNodeId);
  if (!renameLease) return;
  const lease = renameLease;
  sidebar.reveal();
  expand(ancestorsOf(structureNodeId, index.value).map((ancestor) => ancestor.id));
  await nextTick();
  if (renameLease !== lease) return;
  const row = outlineRows.value.findIndex(({ value }) => value.id === structureNodeId);
  if (row >= 0) virtualizer.value?.scrollToIndex(row, { align: "auto" });
  editingId.value = structureNodeId;
}

function finishRename() {
  if (renameLease !== undefined) drafts.end(renameLease);
  renameLease = undefined;
  editingId.value = undefined;
}

async function saveName(structureNodeId: string, title: string) {
  if (editingId.value !== structureNodeId || renameLease === undefined) return;
  const structureNode = index.value.structureById.get(structureNodeId);
  if (!structureNode) throw new Error("Structure node no longer exists");
  if (structureNode.kind === "content") return store.renameNode(structureNode.nodeId, title);
  return store.executeSpaceEdit({ kind: "container.rename", structureNodeId: structureNodeId, title });
}

const outlineMenu = useStructureOutlineMenu({
  index,
  rename,
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
      <section
        class="relative h-full min-w-0 overflow-x-hidden overflow-y-auto"
        :aria-label="t('spaces.sidebar.outline.title')"
      >
        <TreeVirtualizer
          v-if="flattenItems.length"
          :estimate-size="32"
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
