<script setup lang="ts">
import { ChevronDown, ChevronRight } from "@lucide/vue";
import { TreeItem, TreeRoot, TreeVirtualizer } from "reka-ui";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useCanvasEditorSession } from "../canvas/editor-session";
import { openSpaceNode } from "../canvas/selection-actions";
import { indexStructure, structureAncestors } from "../model/structure/structure-index";
import { useSpaceStore } from "../session/space-store";
import SpaceOutlineRow from "./SpaceOutlineRow.vue";
import { projectStructureOutline, type StructureOutlineItem } from "./structure-outline";
import { useStructureMenu } from "./use-structure-menu";

const store = useSpaceStore();
const session = useCanvasEditorSession();
const menu = useStructureMenu();
const { t } = useI18n();
const expanded = shallowRef<string[]>([]);
const editingId = shallowRef<string>();
const structure = computed(() => session.document.value?.structure ?? { version: 1 as const, items: [] });
const projection = computed(() => projectStructureOutline(structure.value, store.nodes));
const selected = computed(() =>
  session.selection.value.nodeIds.flatMap((id) =>
    projection.value.records.has(id) ? [projection.value.records.get(id)!] : [],
  ),
);
let known = new Set<string>();
function select(items: StructureOutlineItem[] | undefined) {
  session.activeEditor.value?.reveal(
    (items ?? []).map((item) => item.id),
    "ensure-visible",
  );
}
function rename(id: string) {
  const tree = indexStructure(structure.value);
  if (!tree.items.has(id)) return;
  expanded.value = [...new Set([...expanded.value, ...structureAncestors(tree, id).map((item) => item.id)])];
  editingId.value = id;
}
async function saveName(item: StructureOutlineItem, title: string) {
  if (item.kind === "content") await store.renameNode(item.id, title);
  else
    await session.activeEditor.value!.executeStructureCommand({
      kind: "rename-container",
      id: item.id,
      title,
    });
}
function open(item: StructureOutlineItem) {
  if (item.kind === "group") session.activeEditor.value?.enterGroup(item.id);
  else if (item.node) void openSpaceNode(store, item.node);
  else session.activeEditor.value?.reveal([item.id], "focus");
}
function toggle(event: CustomEvent<{ originalEvent: Event }>) {
  if (event.detail.originalEvent.type !== "keydown") event.preventDefault();
}
watch(
  () => store.activeSpaceId,
  () => {
    editingId.value = undefined;
    expanded.value = [];
    known = new Set();
  },
);
watch(
  projection,
  ({ records }) => {
    const containers = [...records.values()].filter((item) => item.kind !== "content").map((item) => item.id);
    expanded.value = [
      ...new Set([
        ...expanded.value.filter((id) => records.has(id)),
        ...containers.filter((id) => !known.has(id)),
      ]),
    ];
    known = new Set(containers);
    if (editingId.value && !records.has(editingId.value)) editingId.value = undefined;
  },
  { immediate: true },
);
watch(
  () => store.outlineRenameNodeId,
  (id) => {
    if (!id || !projection.value.records.has(id)) return;
    store.outlineRenameNodeId = null;
    rename(id);
  },
);
</script>
<template>
  <section class="min-w-0" @contextmenu.self.prevent="menu.show({})">
    <TreeRoot
      v-if="projection.roots.length"
      multiple
      :items="projection.roots"
      :get-key="(item: StructureOutlineItem) => item.id"
      :get-children="(item: StructureOutlineItem) => item.children"
      :model-value="selected"
      v-model:expanded="expanded"
      selection-behavior="replace"
      :aria-label="t('spaces.navigation.outline')"
      class="max-h-96 overflow-auto outline-none"
      data-testid="space-outline-tree"
      @update:model-value="select"
    >
      <TreeVirtualizer v-slot="{ item }" :text-content="(item: StructureOutlineItem) => item.title">
        <TreeItem
          v-bind="item.bind"
          v-slot="{ isExpanded, isSelected, handleToggle }"
          :data-item-id="item.value.id"
          class="data-[selected]:bg-control-active flex min-w-0 items-center rounded-control pr-2 text-caption text-foreground-secondary outline-none hover:bg-control-hover focus-visible:ring-1 focus-visible:ring-focus-ring data-[selected]:text-foreground"
          :style="{ paddingLeft: `${(item.level - 1) * 14 + 4}px` }"
          @toggle="toggle"
          @contextmenu.prevent.stop="menu.show({ itemId: item.value.id, rename })"
        >
          <button
            v-if="item.hasChildren"
            type="button"
            tabindex="-1"
            class="grid size-6 shrink-0 place-items-center rounded-control text-foreground-tertiary"
            :aria-label="
              t(isExpanded ? 'spaces.structure.collapseOutline' : 'spaces.structure.expandOutline')
            "
            @click.stop="handleToggle"
          >
            <ChevronDown v-if="isExpanded" class="size-(--knovum-icon-size-xs)" /><ChevronRight
              v-else
              class="size-(--knovum-icon-size-xs)"
            />
          </button>
          <span v-else class="w-6 shrink-0" />
          <SpaceOutlineRow
            :item="item.value"
            :selected="isSelected"
            :editing="editingId === item.value.id"
            :save-name="(title) => saveName(item.value, title)"
            @finish="editingId = undefined"
            @open="open(item.value)"
          />
        </TreeItem>
      </TreeVirtualizer>
    </TreeRoot>
    <p v-else class="mt-2 px-1 text-caption text-foreground-tertiary">
      {{ t("spaces.navigation.outlineEmpty") }}
    </p>
  </section>
</template>
