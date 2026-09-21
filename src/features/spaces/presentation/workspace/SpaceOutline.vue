<script setup lang="ts">
import { ChevronDown, ChevronRight } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { xor } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed, inject, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { openSpaceNode, selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem } from "../../workspace/space-menu";
import SpaceOutlineRow from "./SpaceOutlineRow.vue";

const store = useSpaceStore();
const { nodes } = storeToRefs(store);
const editorSession = useCanvasEditorSession();
const { t } = useI18n();
const collapsed = ref<string[]>([]);
const editingId = shallowRef<string | null>(null);
const sidebar = inject(workspaceSidebarKey)!;
const sections = computed(() => nodes.value.filter((node) => node.data.kind === "section"));
const ungrouped = computed(() =>
  nodes.value.filter((node) => node.data.kind !== "section" && !node.data.sectionId),
);

function members(sectionId: string) {
  return nodes.value.filter((node) => node.data.sectionId === sectionId);
}
function reveal(node: SpaceCanvasNode) {
  selectAndRevealSpaceNode(node.id);
}
function rename(node: SpaceCanvasNode) {
  sidebar.reveal();
  if (node.data.sectionId) collapsed.value = collapsed.value.filter((id) => id !== node.data.sectionId);
  editingId.value = node.id;
}
watch(
  () => store.outlineRenameNodeId,
  (id) => {
    if (id === null) return;
    store.outlineRenameNodeId = null;
    const node = nodes.value.find((candidate) => candidate.id === id);
    if (node) rename(node);
  },
  { immediate: true },
);
watch(
  () => store.activeSpaceId,
  () => {
    editingId.value = null;
  },
);
watch(nodes, () => {
  if (!nodes.value.some((node) => node.id === editingId.value)) editingId.value = null;
});
async function deleteSectionContents(sectionId: string) {
  const confirmed = await ask(t("spaces.canvas.deleteSectionContentsDescription"), {
    title: t("spaces.canvas.deleteSectionWithContents"),
    kind: "warning",
    okLabel: t("common.delete"),
    cancelLabel: t("common.cancel"),
  });
  if (confirmed) await store.deleteSectionWithContents(sectionId);
}

async function showNodeMenu(node: SpaceCanvasNode) {
  if (!editorSession.selection.value.nodeIds.includes(node.id)) editorSession.selectNodes([node.id]);
  const open =
    node.data.kind === "section"
      ? spaceMenuItem(t("spaces.canvas.revealInCanvas"), () => reveal(node))
      : spaceMenuItem(t("common.open"), () => openSpaceNode(store, node));
  const section =
    node.data.kind === "section"
      ? [
          spaceMenuItem(
            node.data.collapsed ? t("spaces.canvas.expandSection") : t("spaces.canvas.collapseSection"),
            () => store.setSectionCollapsed(node.id, node.data.kind === "section" && !node.data.collapsed),
          ),
        ]
      : [];
  const state = [
    spaceMenuItem(node.data.locked ? t("spaces.canvas.unlock") : t("spaces.canvas.lock"), () =>
      store.setLocked([node.id], !node.data.locked),
    ),
  ];
  const remove =
    node.data.kind === "section"
      ? [
          spaceMenuItem(t("spaces.canvas.removeSection"), () => store.removeSection(node.id)),
          spaceMenuItem(t("spaces.canvas.deleteSectionWithContents"), () => deleteSectionContents(node.id)),
        ]
      : [spaceMenuItem(t("spaces.navigation.deleteSelection"), () => store.deleteSelection([node.id]))];
  await popupSpaceContextMenu([
    [open, spaceMenuItem(t("spaces.navigation.renameNode"), () => rename(node))],
    section,
    state,
    remove,
  ]);
}

function selected(id: string) {
  return editorSession.selection.value.nodeIds.includes(id);
}
function toggleOutline(id: string) {
  collapsed.value = xor(collapsed.value, [id]);
}
</script>

<template>
  <section>
    <div v-if="sections.length" class="mt-2 space-y-0.5">
      <div v-for="section in sections" :key="section.id">
        <div
          class="flex items-center rounded-control hover:bg-control-hover"
          :data-active="selected(section.id)"
        >
          <button
            type="button"
            class="grid size-6 shrink-0 place-items-center rounded-control text-foreground-tertiary hover:text-foreground"
            :aria-label="
              collapsed.includes(section.id)
                ? t('spaces.canvas.expandSection')
                : t('spaces.canvas.collapseSection')
            "
            :aria-expanded="!collapsed.includes(section.id)"
            @click="toggleOutline(section.id)"
          >
            <ChevronRight v-if="collapsed.includes(section.id)" class="size-(--knovum-icon-size-xs)" />
            <ChevronDown v-else class="size-(--knovum-icon-size-xs)" />
          </button>
          <SpaceOutlineRow
            class="min-w-0 flex-1 pr-2 text-label text-foreground-secondary data-[active=true]:text-foreground"
            :node="section"
            :selected="selected(section.id)"
            :editing="editingId === section.id"
            :save-name="(title) => store.renameNode(section.id, title)"
            @reveal="reveal(section)"
            @open="reveal(section)"
            @menu="showNodeMenu(section)"
            @finish="editingId = null"
          >
            <span class="text-micro text-foreground-tertiary tabular-nums">{{
              members(section.id).length
            }}</span>
          </SpaceOutlineRow>
        </div>
        <div class="mt-0.5 space-y-0.5">
          <SpaceOutlineRow
            v-for="node in collapsed.includes(section.id) ? [] : members(section.id)"
            :key="node.id"
            class="w-full rounded-control pr-2 pl-7 text-caption text-foreground-secondary hover:bg-control-hover hover:text-foreground data-[active=true]:bg-(--knovum-sidebar-item-background-selected) data-[active=true]:text-foreground"
            :node="node"
            :selected="selected(node.id)"
            :editing="editingId === node.id"
            :save-name="(title) => store.renameNode(node.id, title)"
            @reveal="reveal(node)"
            @open="openSpaceNode(store, node)"
            @menu="showNodeMenu(node)"
            @finish="editingId = null"
          />
        </div>
      </div>
    </div>
    <div v-if="ungrouped.length" class="mt-2 space-y-0.5">
      <p
        v-if="sections.length"
        class="px-2 py-1 text-micro font-medium tracking-uppercase text-foreground-tertiary uppercase"
      >
        {{ t("spaces.canvas.ungrouped") }}
      </p>
      <SpaceOutlineRow
        v-for="node in ungrouped"
        :key="node.id"
        class="w-full rounded-control px-2 text-caption text-foreground-secondary hover:bg-control-hover hover:text-foreground data-[active=true]:bg-(--knovum-sidebar-item-background-selected) data-[active=true]:text-foreground"
        :node="node"
        :selected="selected(node.id)"
        :editing="editingId === node.id"
        :save-name="(title) => store.renameNode(node.id, title)"
        @reveal="reveal(node)"
        @open="openSpaceNode(store, node)"
        @menu="showNodeMenu(node)"
        @finish="editingId = null"
      />
    </div>
    <p v-if="nodes.length === 0" class="mt-2 px-1 text-caption text-foreground-tertiary">
      {{ t("spaces.navigation.outlineEmpty") }}
    </p>
    <p
      v-else-if="!sections.length && !ungrouped.length"
      class="mt-2 px-1 text-caption text-foreground-tertiary"
    >
      {{ t("spaces.navigation.noOutlineResults") }}
    </p>
  </section>
</template>
