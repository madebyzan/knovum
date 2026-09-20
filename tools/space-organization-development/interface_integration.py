"""Migrate product selection, Outline, navigation, and context menus to structure."""
from pathlib import Path
import json
import re

root = Path.cwd()
marker = root / "tools/space-organization-development/applied/interface-integration"
if marker.exists():
    raise SystemExit(0)
sources = {}


def read(path):
    return sources[path] if path in sources else (root / path).read_text()


def write(path, text):
    sources[path] = text.strip() + "\n"


def replace(path, before, after):
    source = read(path)
    if before not in source:
        raise RuntimeError(f"Reviewed source precondition failed: {path}: {before[:100]}")
    write(path, source.replace(before, after))


write("src/features/spaces/presentation/canvas/selection-product-state.ts", r'''
import { isSpaceNodeOpenable } from "../../canvas/selection-actions";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import type { SpaceStructure } from "../../model/structure/structure-schema";
import { indexStructure } from "../../model/structure/structure-index";
import { structureContentIds } from "../../organization/structure/structure-selection";
import { structureSelectionEligibility } from "../../organization/structure/structure-eligibility";

export function canvasSelectionProductState(nodes: readonly SpaceCanvasNode[], selectedIds: readonly string[], structure: SpaceStructure) {
  const state = structureSelectionEligibility(structure, selectedIds);
  const selected = new Set(state.roots.map((item) => item.id));
  const directContent = nodes.filter((node) => selected.has(node.id));
  const contentIds = new Set(structureContentIds(indexStructure(structure), [...selected]));
  const content = nodes.filter((node) => contentIds.has(node.id));
  const single = state.roots.length === 1 ? state.roots[0] : undefined;
  const section = single?.kind === "section" ? single : undefined;
  const group = single?.kind === "group" ? single : undefined;
  const data = single?.kind === "content" ? directContent[0]?.data : undefined;
  const textKind = data?.kind === "note" || data?.kind === "text" ? data.kind : undefined;
  return { ...state, section, group, content, directContent, textKind,
    allLocked: state.roots.length > 0 && state.roots.every((item) => item.locked),
    askScope: section ? "section" as const : "selection" as const,
    canConnect: state.editable && state.roots.length === 2 && directContent.length === 2 && directContent.every(isSpaceSemanticNode),
    canOpen: single?.kind === "content" && isSpaceNodeOpenable(directContent[0]),
    opensSystemPreview: Boolean(data && isSpaceManagedAssetPayload(data)),
    hasParent: state.roots.some((item) => item.parentId !== null),
  };
}
''')
write("src/features/spaces/presentation/canvas/use-canvas-selection-actions.ts", r'''
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { CanvasNodeAppearance } from "../../canvas/contract/canvas-document";
import type { CanvasAlignMode, CanvasAutoLayoutDirection, CanvasDistributeMode, CanvasLayerMove } from "../../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { openSpaceNode } from "../../canvas/selection-actions";
import { useContextDock } from "../../canvas/use-context-dock";
import type { StructureCommand } from "../../organization/structure/commands/contract";
import { useSpaceStore } from "../../session/space-store";
import { canvasSelectionProductState } from "./selection-product-state";

function run(operation: () => Promise<unknown>) { void Promise.resolve().then(operation).catch((error) => toast.error(String(error))); }

export function useCanvasSelectionActions() {
  const store = useSpaceStore();
  const dock = useContextDock();
  const session = useCanvasEditorSession();
  const { t } = useI18n();
  const structure = computed(() => session.document.value?.structure ?? { version: 1 as const, items: [] });
  const state = computed(() => canvasSelectionProductState(store.nodes, session.selection.value.nodeIds, structure.value));
  const selectedNodes = computed(() => state.value.directContent);
  const selectedAppearance = computed(() => session.selection.value.singleNodeAppearance);
  const visible = computed(() => state.value.roots.length > 0 || session.selection.value.relationIds.length > 0);
  const ids = () => state.value.roots.map((item) => item.id);
  function command(command: StructureCommand) {
    const editor = session.activeEditor.value;
    if (editor) run(() => editor.executeStructureCommand(command));
  }
  function createSection() {
    const editor = session.activeEditor.value;
    if (!editor || !state.value.canSection) return;
    const id = crypto.randomUUID();
    run(async () => {
      await editor.executeStructureCommand({ kind: "section-around-selection", id, title: t("spaces.structure.untitledSection"), ids: ids() });
      editor.requestContainerRename(id);
    });
  }
  function group() { command({ kind: "group-selection", id: crypto.randomUUID(), title: t("spaces.structure.untitledGroup"), ids: ids() }); }
  function ungroup() { if (state.value.group) command({ kind: "dissolve-container", id: state.value.group.id }); }
  function removeSection() { if (state.value.section) command({ kind: "dissolve-container", id: state.value.section.id }); }
  function fitSection() { if (state.value.section) command({ kind: "fit-section", id: state.value.section.id }); }
  function setAppearance(patch: Partial<CanvasNodeAppearance>) {
    const editor = session.activeEditor.value;
    const [id] = ids();
    if (editor && id && state.value.roots.length === 1) run(() => editor.setNodeAppearance(id, patch));
  }
  function connect() {
    const [source, target] = state.value.directContent;
    if (state.value.canConnect && source && target) run(() => store.createRelation(source.id, target.id, "related"));
  }
  function focus() {
    if (state.value.section) store.focusSection(state.value.section.id);
    else if (state.value.group) store.focusItems([state.value.group.id]);
    else if (state.value.directContent[0]) store.focusNeighborhood(state.value.directContent[0].id);
  }
  function editorAction(action: (editor: NonNullable<typeof session.activeEditor.value>) => Promise<unknown>) {
    const editor = session.activeEditor.value;
    if (editor) run(() => action(editor));
  }
  return {
    state, selectedNodes, selectedAppearance, visible, setAppearance, connect, focus, group, ungroup, createSection, fitSection, removeSection,
    inspect: () => dock.openInspect(), ask: () => dock.openAsk(state.value.askScope, { restart: true }),
    open: () => run(() => openSpaceNode(store, state.value.directContent[0])),
    duplicate: () => editorAction((editor) => editor.duplicateSelection()),
    remove: () => editorAction((editor) => editor.deleteSelection()),
    toggleLock: () => command({ kind: "set-items-locked", ids: ids(), locked: !state.value.allLocked }),
    moveToRoot: () => command({ kind: "reparent-items", ids: ids(), parentId: null }),
    moveToSpace: (spaceId: string) => run(() => store.moveNodes(ids(), spaceId)),
    align: (mode: CanvasAlignMode) => editorAction((editor) => editor.alignSelection(mode)),
    distribute: (mode: CanvasDistributeMode) => editorAction((editor) => editor.distributeSelection(mode)),
    autoLayout: (direction: CanvasAutoLayoutDirection) => editorAction((editor) => editor.autoLayoutSelection(direction)),
    moveLayer: (mode: CanvasLayerMove) => editorAction((editor) => editor.moveSelectionLayer(mode)),
  };
}
export type CanvasSelectionActions = ReturnType<typeof useCanvasSelectionActions>;
''')
write("src/features/spaces/presentation/canvas/SpaceCanvasSelectionBar.vue", r'''
<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { useI18n } from "vue-i18n";
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import type { CreationIntentDecisions, CreationIntentId } from "../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceCanvasSelectionPrimary from "./SpaceCanvasSelectionPrimary.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import { useCanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ derivations: CreationIntentDecisions }>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const actions = useCanvasSelectionActions();
const { t } = useI18n();
</script>
<template>
  <ToolbarRoot v-if="actions.visible.value" orientation="horizontal" :loop="false"
    :aria-label="t('spaces.navigation.selection')"
    class="flex h-7 cursor-default items-center gap-0.5 rounded-control border border-border bg-overlay select-none">
    <UiBadge v-if="actions.state.value.roots.length > 1" size="sm" class="mx-1">{{ actions.state.value.roots.length }}</UiBadge>
    <SpaceCanvasSelectionPrimary :actions="actions" />
    <SpaceCanvasSelectionMenu :actions="actions" :derivations="derivations" @invoke-derivation="emit('invokeDerivation', $event)" />
  </ToolbarRoot>
</template>
''')
write("src/features/spaces/presentation/canvas/SpaceCanvasSelectionPrimary.vue", r'''
<script setup lang="ts">
import { Copy, Eye, ExternalLink, Frame, Group, Info, Link2, Sparkles, Ungroup } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import SpaceSelectionMoveMenu from "../SpaceSelectionMoveMenu.vue";
import SpaceCanvasTextFormatBar from "./SpaceCanvasTextFormatBar.vue";
import type { CanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ actions: CanvasSelectionActions }>();
const { t } = useI18n();
</script>
<template>
  <SpaceCanvasTextFormatBar v-if="actions.state.value.textKind && actions.state.value.editable"
    :kind="actions.state.value.textKind" v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}" @update="actions.setAppearance" />
  <ToolbarButton v-if="actions.state.value.canOpen" as-child>
    <UiIconButton :label="t(actions.state.value.opensSystemPreview ? 'spaces.transfer.systemPreview' : 'common.open')" size="sm" @click="actions.open">
      <Eye v-if="actions.state.value.opensSystemPreview" class="size-(--knovum-icon-size-sm)" /><ExternalLink v-else class="size-(--knovum-icon-size-sm)" />
    </UiIconButton>
  </ToolbarButton>
  <ToolbarButton as-child><UiIconButton :label="t('spaces.navigation.inspect')" size="sm" @click="actions.inspect"><Info class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.canConnect" as-child><UiIconButton :label="t('spaces.canvas.connect')" size="sm" @click="actions.connect"><Link2 class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.content.length" as-child><UiIconButton :label="t(actions.state.value.section ? 'spaces.assistant.askSection' : 'spaces.assistant.askSelection')" size="sm" @click="actions.ask"><Sparkles class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.roots.length" as-child><UiIconButton :label="t('spaces.navigation.duplicateSelection')" shortcut="$mod+D" size="sm" @click="actions.duplicate"><Copy class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton>
  <SpaceSelectionMoveMenu v-if="actions.state.value.editable" @move="actions.moveToSpace" />
  <ToolbarButton v-if="actions.state.value.canGroup" as-child><UiIconButton :label="t('spaces.structure.groupSelection')" shortcut="$mod+G" size="sm" @click="actions.group"><Group class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.canUngroup" as-child><UiIconButton :label="t('spaces.structure.ungroup')" shortcut="$mod+Shift+G" size="sm" @click="actions.ungroup"><Ungroup class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.canSection" as-child><UiIconButton :label="t('spaces.structure.sectionFromSelection')" size="sm" @click="actions.createSection"><Frame class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton>
</template>
''')
write("src/features/spaces/presentation/canvas/SpaceCanvasSelectionMenu.vue", r'''
<script setup lang="ts">
import { Focus, Frame, LockKeyhole, MoreHorizontal, Trash2, UnlockKeyhole } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type { CreationIntentDecisions, CreationIntentId } from "../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceDerivationMenuItems from "../SpaceDerivationMenuItems.vue";
import SpaceCanvasArrangeMenu from "./SpaceCanvasArrangeMenu.vue";
import SpaceCanvasLayerMenu from "./SpaceCanvasLayerMenu.vue";
import type { CanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ actions: CanvasSelectionActions; derivations: CreationIntentDecisions }>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const menuRow = "flex h-(--knovum-compact-item-height) cursor-default items-center gap-2 rounded-control px-2 text-label text-foreground outline-none data-[highlighted]:bg-control-hover data-[disabled]:text-foreground-tertiary";
const submenu = "material-overlay z-50 min-w-48 rounded-panel p-1 ring-1 ring-border";
</script>
<template>
  <UiDropdownMenu side="top" align="center" :side-offset="6" class="min-w-48">
    <template #trigger><ToolbarButton as-child><UiIconButton :label="t('common.more')" size="sm"><MoreHorizontal class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton></template>
    <template v-if="actions.state.value.roots.length">
      <UiDropdownMenuItem @select="actions.focus"><Focus class="size-(--knovum-icon-size-sm)" />{{ t('spaces.navigation.focusSelection') }}</UiDropdownMenuItem>
      <SpaceDerivationMenuItems :derivations="derivations" :row-class="menuRow" :submenu-class="submenu" @invoke="emit('invokeDerivation', $event)" />
      <UiDropdownMenuItem @select="actions.toggleLock">
        <UnlockKeyhole v-if="actions.state.value.allLocked" class="size-(--knovum-icon-size-sm)" /><LockKeyhole v-else class="size-(--knovum-icon-size-sm)" />
        {{ t(actions.state.value.allLocked ? 'spaces.structure.unlock' : 'spaces.structure.lock') }}
      </UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="actions.state.value.section" :disabled="!actions.state.value.editable || !actions.state.value.content.length" @select="actions.fitSection"><Frame class="size-(--knovum-icon-size-sm)" />{{ t('spaces.structure.fit') }}</UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="actions.state.value.hasParent" :disabled="!actions.state.value.editable" @select="actions.moveToRoot">{{ t('spaces.structure.moveToRoot') }}</UiDropdownMenuItem>
      <SpaceCanvasArrangeMenu v-if="actions.state.value.editable && actions.state.value.roots.length > 1" :count="actions.state.value.roots.length" :row-class="menuRow" :submenu-class="submenu" @align="actions.align" @auto-layout="actions.autoLayout" @distribute="actions.distribute" />
      <SpaceCanvasLayerMenu v-if="actions.state.value.editable" :row-class="menuRow" :submenu-class="submenu" @move="actions.moveLayer" />
      <UiDropdownMenuItem v-if="actions.state.value.section" :disabled="!actions.state.value.editable" @select="actions.removeSection">{{ t('spaces.structure.dissolveSection') }}</UiDropdownMenuItem>
      <UiSeparator class="my-1" />
    </template>
    <UiDropdownMenuItem destructive :disabled="actions.state.value.roots.length > 0 && !actions.state.value.editable" @select="actions.remove"><Trash2 class="size-(--knovum-icon-size-sm)" />{{ t('spaces.navigation.deleteSelection') }}</UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
''')
write("src/features/spaces/presentation/structure-outline.ts", r'''
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { indexStructure, structureAncestors } from "../model/structure/structure-index";
import type { SpaceStructure, StructureItem } from "../model/structure/structure-schema";

export interface StructureOutlineItem {
  id: string; title: string; kind: StructureItem["kind"]; locked: boolean;
  node?: SpaceCanvasNode;
  children?: StructureOutlineItem[];
}
export function projectStructureOutline(structure: SpaceStructure, nodes: readonly SpaceCanvasNode[]) {
  const tree = indexStructure(structure);
  const content = new Map(nodes.map((node) => [node.id, node]));
  const records = new Map<string, StructureOutlineItem>();
  for (const item of structure.items) {
    const node = item.kind === "content" ? content.get(item.nodeId) : undefined;
    if (item.kind === "content" && !node) continue;
    records.set(item.id, { id: item.id, kind: item.kind,
      title: item.kind === "content" ? node!.data.title : item.title,
      locked: item.locked || structureAncestors(tree, item.id).some((parent) => parent.locked),
      ...(node ? { node } : {}),
    });
  }
  for (const [parentId, children] of tree.children) {
    if (parentId === null) continue;
    const parent = records.get(parentId);
    const available = children.flatMap(({ id }) => records.has(id) ? [records.get(id)!] : []);
    if (parent && available.length) parent.children = available;
  }
  return { records, roots: (tree.children.get(null) ?? []).flatMap(({ id }) => records.has(id) ? [records.get(id)!] : []) };
}
''')
write("src/features/spaces/presentation/SpaceOutlineRow.vue", r'''
<script setup lang="ts">
import { Frame, Group, LockKeyhole } from "@lucide/vue";
import { nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { StructureOutlineItem } from "./structure-outline";
import { nodeIcon } from "./node-icon";
import SpaceOutlineTitle from "./SpaceOutlineTitle.vue";

const props = defineProps<{ item: StructureOutlineItem; selected: boolean; editing: boolean; saveName(title: string): Promise<void> }>();
const emit = defineEmits<{ open: []; finish: [] }>();
const { t } = useI18n();
const input = useTemplateRef<HTMLInputElement>("input");
const row = useTemplateRef<HTMLElement>("row");
const draft = shallowRef(""); const saving = shallowRef(false); const error = shallowRef("");
watch(() => props.editing, async (editing) => {
  if (!editing) return;
  draft.value = props.item.title; error.value = "";
  await nextTick();
  if (!props.editing) return;
  input.value?.scrollIntoView({ block: "nearest" }); input.value?.focus({ preventScroll: true }); input.value?.select();
}, { immediate: true });
async function finish(restoreFocus: boolean) {
  emit("finish");
  if (restoreFocus) { await nextTick(); row.value?.closest<HTMLElement>('[role="treeitem"]')?.focus({ preventScroll: true }); }
}
async function save() {
  if (saving.value) return;
  const title = draft.value.trim();
  if (!title || title.length > 512) { error.value = t("spaces.structure.renameInvalid"); return; }
  saving.value = true; error.value = "";
  try { await props.saveName(title); await finish(true); }
  catch { error.value = t("spaces.structure.saveFailed"); await nextTick(); input.value?.focus(); }
  finally { saving.value = false; }
}
function keydown(event: KeyboardEvent) {
  event.stopPropagation();
  if (event.isComposing) return;
  if (event.key === "Enter") { event.preventDefault(); void save(); }
  if (event.key === "Escape" && !saving.value) { event.preventDefault(); void finish(true); }
}
</script>
<template>
  <div ref="row" class="min-w-0 flex-1" :data-active="selected">
    <div class="flex h-(--knovum-compact-item-height) min-w-0 items-center gap-2" @dblclick="!editing && emit('open')">
      <component :is="item.kind === 'section' ? Frame : item.kind === 'group' ? Group : nodeIcon(item.node!.data.kind)" class="size-(--knovum-icon-size-xs) shrink-0" />
      <input v-if="editing" ref="input" v-model="draft" :maxlength="512" :readonly="saving" :aria-label="t('spaces.structure.rename')" :aria-invalid="Boolean(error)" :aria-busy="saving"
        class="h-6 min-w-0 flex-1 rounded-control bg-control px-1 text-caption text-foreground ring-1 ring-focus-ring outline-none"
        @keydown="keydown" @pointerdown.stop @contextmenu.stop @blur="!saving && !error && finish(false)" />
      <SpaceOutlineTitle v-else :title="item.title" />
      <LockKeyhole v-if="item.locked" class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
    </div>
    <p v-if="editing && error" role="alert" class="pb-1 text-micro text-danger">{{ error }}</p>
  </div>
</template>
''')
write("src/features/spaces/presentation/SpaceOutline.vue", r'''
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

const store = useSpaceStore(); const session = useCanvasEditorSession(); const menu = useStructureMenu();
const { t } = useI18n();
const expanded = shallowRef<string[]>([]); const editingId = shallowRef<string>();
const structure = computed(() => session.document.value?.structure ?? { version: 1 as const, items: [] });
const projection = computed(() => projectStructureOutline(structure.value, store.nodes));
const selected = computed(() => session.selection.value.nodeIds.flatMap((id) => projection.value.records.has(id) ? [projection.value.records.get(id)!] : []));
let known = new Set<string>();
function select(items: StructureOutlineItem[] | undefined) { session.activeEditor.value?.reveal((items ?? []).map((item) => item.id), "ensure-visible"); }
function rename(id: string) {
  const tree = indexStructure(structure.value);
  if (!tree.items.has(id)) return;
  expanded.value = [...new Set([...expanded.value, ...structureAncestors(tree, id).map((item) => item.id)])];
  editingId.value = id;
}
async function saveName(item: StructureOutlineItem, title: string) {
  if (item.kind === "content") await store.renameNode(item.id, title);
  else await session.activeEditor.value!.executeStructureCommand({ kind: "rename-container", id: item.id, title });
}
function open(item: StructureOutlineItem) {
  if (item.kind === "group") session.activeEditor.value?.enterGroup(item.id);
  else if (item.node) void openSpaceNode(store, item.node);
  else session.activeEditor.value?.reveal([item.id], "focus");
}
function toggle(event: CustomEvent<{ originalEvent: Event }>) {
  if (event.detail.originalEvent.type !== "keydown") event.preventDefault();
}
watch(() => store.activeSpaceId, () => { editingId.value = undefined; expanded.value = []; known = new Set(); });
watch(projection, ({ records }) => {
  const containers = [...records.values()].filter((item) => item.kind !== "content").map((item) => item.id);
  expanded.value = [...new Set([...expanded.value.filter((id) => records.has(id)), ...containers.filter((id) => !known.has(id))])];
  known = new Set(containers);
  if (editingId.value && !records.has(editingId.value)) editingId.value = undefined;
}, { immediate: true });
watch(() => store.outlineRenameNodeId, (id) => {
  if (!id || !projection.value.records.has(id)) return;
  store.outlineRenameNodeId = null; rename(id);
});
</script>
<template>
  <section class="min-w-0" @contextmenu.self.prevent="menu.show({})">
    <TreeRoot v-if="projection.roots.length" multiple :items="projection.roots" :get-key="(item: StructureOutlineItem) => item.id"
      :get-children="(item: StructureOutlineItem) => item.children" :model-value="selected" v-model:expanded="expanded"
      selection-behavior="replace" :aria-label="t('spaces.navigation.outline')" class="max-h-96 overflow-auto outline-none" data-testid="space-outline-tree" @update:model-value="select">
      <TreeVirtualizer v-slot="{ item }" :text-content="(item: StructureOutlineItem) => item.title">
        <TreeItem v-bind="item.bind" v-slot="{ isExpanded, isSelected, handleToggle }" :data-item-id="item.value.id"
          class="flex min-w-0 items-center rounded-control pr-2 text-caption text-foreground-secondary outline-none hover:bg-control-hover focus-visible:ring-1 focus-visible:ring-focus-ring data-[selected]:bg-control-active data-[selected]:text-foreground"
          :style="{ paddingLeft: `${(item.level - 1) * 14 + 4}px` }" @toggle="toggle" @contextmenu.prevent.stop="menu.show({ itemId: item.value.id, rename })">
          <button v-if="item.hasChildren" type="button" tabindex="-1" class="grid size-6 shrink-0 place-items-center rounded-control text-foreground-tertiary"
            :aria-label="t(isExpanded ? 'spaces.structure.collapseOutline' : 'spaces.structure.expandOutline')" @click.stop="handleToggle">
            <ChevronDown v-if="isExpanded" class="size-(--knovum-icon-size-xs)" /><ChevronRight v-else class="size-(--knovum-icon-size-xs)" />
          </button>
          <span v-else class="w-6 shrink-0" />
          <SpaceOutlineRow :item="item.value" :selected="isSelected" :editing="editingId === item.value.id" :save-name="(title) => saveName(item.value, title)" @finish="editingId = undefined" @open="open(item.value)" />
        </TreeItem>
      </TreeVirtualizer>
    </TreeRoot>
    <p v-else class="mt-2 px-1 text-caption text-foreground-tertiary">{{ t('spaces.navigation.outlineEmpty') }}</p>
  </section>
</template>
''')
write("src/features/spaces/presentation/use-structure-menu.ts", r'''
import { useI18n } from "vue-i18n";
import type { CanvasPoint } from "../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { structureSelectionEligibility } from "../organization/structure/structure-eligibility";
import { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem, spaceSubmenu } from "../workspace/space-menu";
import { useCanvasSelectionActions } from "./canvas/use-canvas-selection-actions";

export function useStructureMenu() {
  const session = useCanvasEditorSession(); const store = useSpaceStore();
  const actions = useCanvasSelectionActions(); const { t } = useI18n();
  async function show(request: { itemId?: string; relationId?: string; point?: CanvasPoint; rename?(id: string): void }) {
    const editor = session.activeEditor.value;
    if (!editor) return;
    if (request.itemId && !session.selection.value.nodeIds.includes(request.itemId)) editor.selectNodes([request.itemId]);
    if (!request.itemId && !request.relationId) editor.clearSelection();
    const document = editor.getDocument(); const ids = [...editor.getSelectionSnapshot().nodeIds];
    const state = structureSelectionEligibility(document.structure, ids);
    const item = state.roots.length === 1 ? state.roots[0] : undefined;
    const label = (key: string) => t(`spaces.structure.${key}`);
    const entry = spaceMenuItem;
    const create = entry(label("newSection"), async () => {
      const id = crypto.randomUUID();
      await editor.executeStructureCommand({ kind: "create-empty-section", id, title: label("untitledSection"), bounds: { ...(request.point ?? store.nextInsertionPoint()), width: 640, height: 400 } });
      editor.requestContainerRename(id);
    });
    await popupSpaceContextMenu([
      ids.length ? [entry(t("common.copy"), () => editor.copySelection()), entry(t("common.cut"), () => editor.cutSelection(), { enabled: state.editable }), entry(t("spaces.navigation.duplicateSelection"), () => editor.duplicateSelection())] : request.relationId ? [] : [create, entry(t("common.paste"), async () => { await editor.pasteSelection(); })],
      ids.length ? [entry(label("groupSelection"), actions.group, { enabled: state.canGroup }), entry(label("ungroup"), actions.ungroup, { enabled: state.canUngroup }), entry(label("sectionFromSelection"), actions.createSection, { enabled: state.canSection })] : [],
      item ? [entry(label("rename"), () => {
        if (request.rename) request.rename(item.id);
        else if (item.kind !== "content") editor.requestContainerRename(item.id);
        else store.outlineRenameNodeId = item.id;
      }, { enabled: state.editable }),
      ...(item.kind === "group" ? [entry(label("enterGroup"), () => editor.enterGroup(item.id), { enabled: state.editable })] : []),
      ...(item.kind === "section" ? [entry(label("fit"), () => editor.fitSection(item.id), { enabled: state.editable }), entry(label("dissolveSection"), actions.removeSection, { enabled: state.editable })] : []),
      entry(item.locked ? label("unlock") : label("lock"), () => editor.executeStructureCommand({ kind: "set-items-locked", ids: [item.id], locked: !item.locked }))] : [],
      state.editable && state.roots.every((item) => item.kind !== "section") ? [
        spaceSubmenu(label("moveToSection"), document.structure.items.filter((item) => item.kind === "section").map((section) => entry(section.title, () => editor.executeStructureCommand({ kind: "reparent-items", ids, parentId: section.id }), { enabled: !section.locked }))),
        entry(label("moveToRoot"), () => editor.executeStructureCommand({ kind: "reparent-items", ids, parentId: null }),
      ] : [],
      ids.length || request.relationId ? [entry(label("deleteSelection"), () => editor.deleteSelection(), { enabled: !ids.length || state.editable })] : [],
    ]);
  }
  return { show };
}
''')
write("src/features/spaces/canvas/konva/viewport/structure-navigation.ts", r'''
import { nextTick } from "vue";
import { indexStructure, structureAncestors } from "../../../model/structure/structure-index";
import type { KonvaEditorPortOptions } from "../canvas-editor-port-options";
import { revealCanvasLayouts } from "./canvas-editor-navigation";

export function createStructureNavigation(options: KonvaEditorPortOptions) {
  function select(ids: readonly string[]) { options.setEditingScope(ids); options.setSelection(ids); }
  async function revealItems(ids: readonly string[], mode: "ensure-visible" | "focus") {
    if (!ids.length) { select([]); return; }
    const document = options.model.document;
    const tree = indexStructure(document.structure);
    const existing = ids.filter((id) => tree.items.has(id));
    const ancestors = new Set(existing.flatMap((id) => structureAncestors(tree, id).filter((item) => item.kind === "section").map((item) => item.id)));
    for (const layout of document.layout.items) {
      if (layout.kind !== "section" || !layout.collapsed || !ancestors.has(layout.itemId)) continue;
      // Reveal changes presentation only; every write still uses a native command and revision.
      // oxlint-disable-next-line no-await-in-loop -- Each command adopts its committed revision before the next reveal.
      await options.model.execute({ kind: "set-section-collapsed", id: layout.itemId, collapsed: false }, [], false);
    }
    options.setEditingScope(existing); await nextTick();
    revealCanvasLayouts({ ...options, setViewport: options.animateViewport }, existing, mode);
  }
  return { select, reveal: (ids: readonly string[], mode: "ensure-visible" | "focus") => { void revealItems(ids, mode).catch(options.onError); } };
}
''')
write("src/features/spaces/session/actions/view-actions.ts", r'''
import type { SpacePlacement } from "../../canvas/navigation-requests";
import { currentCanvasEditor } from "../../canvas/editor-session";
import { indexStructure, structureAncestors, structureSubtree } from "../../model/structure/structure-index";
import type { SpaceSavedViewFilter } from "../../knowledge/model/projections";
import { snapshotSpaceQueryFilter } from "../../knowledge/model/query-filter";
import { findSemanticRelationPath, queryMatchingNodes, relationNeighborhood } from "../../knowledge/model/relation-graph";
import type { SpaceState } from "../session-state";

export function createSpaceViewActions(state: SpaceState, placement: SpacePlacement) {
  function focusItems(ids: readonly string[]) {
    const document = currentCanvasEditor()?.getDocument();
    if (!document) return;
    const tree = indexStructure(document.structure);
    const selected = structureSubtree(tree, ids.filter((id) => tree.items.has(id))).map((item) => item.id);
    state.canvasFocusIds.value = includeContainers(new Set(selected));
  }
  function focusNeighborhood(id: string, depth = 1) { state.canvasFocusIds.value = includeContainers(relationNeighborhood(state.relations.value, [id], depth)); }
  function focusRelationPath(sourceId: string, targetId: string) {
    const path = findSemanticRelationPath(state.relations.value, sourceId, targetId);
    if (!path.length) return false;
    state.canvasFocusIds.value = includeContainers(new Set(path)); return true;
  }
  async function openSceneView(id: string) {
    const scene = state.scenes.value.find((candidate) => candidate.id === id);
    if (!scene) return;
    state.relationKindFilter.value = [...scene.relationKinds];
    state.relationStatusFilter.value = [...scene.relationStatuses];
    state.savedQueryFilter.value = snapshotSpaceQueryFilter(scene.queryFilter);
    const dynamic = hasQueryFilter(scene.queryFilter) ? queryMatchingNodes(state.nodes.value, state.relations.value, scene.queryFilter).map((node) => node.id) : scene.focusNodeIds;
    if (dynamic.length) focusItems(dynamic); else state.canvasFocusIds.value = [];
    placement.requestViewport({ x: scene.viewportX, y: scene.viewportY, zoom: scene.viewportZoom });
    placement.announceScene(scene, "opened");
  }
  return { focusItems, focusSection: (id: string) => focusItems([id]), focusNeighborhood, focusRelationPath,
    clearCanvasFocus: () => { state.canvasFocusIds.value = []; }, openSceneView };
}
function includeContainers(ids: Set<string>) {
  const document = currentCanvasEditor()?.getDocument();
  if (document) {
    const tree = indexStructure(document.structure);
    for (const id of [...ids]) if (tree.items.has(id)) for (const parent of structureAncestors(tree, id)) ids.add(parent.id);
  }
  return [...ids];
}
function hasQueryFilter(filter: SpaceSavedViewFilter) {
  return Object.values(filter).some((value) => Array.isArray(value) ? value.length > 0 : value !== undefined && value !== "");
}
''')
# Preserve independent Outline expansion; tag navigation now uses the shared reveal policy.
path = "src/features/spaces/presentation/SpaceTags.vue"
source = read(path).replace("compact, countBy, take, uniq", "countBy, take")
source = source.replace('selectedNodes.value.filter((node) => node.data.kind !== "section")', "selectedNodes.value")
source = source.replace('nodes.value.filter((node) => node.data.kind !== "section").flatMap', "nodes.value.flatMap")
source = source.replace('node.data.kind !== "section" && ', "")
start = source.index("  const sectionIds =")
end = source.index("  if (store.activeSpaceId", start)
source = source[:start] + source[end:]
write(path, source)
path = "src/features/spaces/commands/use-canvas-navigation-commands.ts"
source = read(path)
source = source.replace('const section = nodes.find(({ data }) => data.kind === "section");', 'const section = session.document.value?.structure.items.find((item) => item.kind === "section" && session.selection.value.nodeIds.includes(item.id));')
source = source.replace('isEnabled: () => selectedNodes().length > 0,', 'isEnabled: () => session.selection.value.nodeIds.length > 0,')
source = source.replace('''        const nodes = selectedNodes();
        await store.setLocked(
          nodes.map(({ id }) => id),
          !nodes.every(({ data }) => Boolean(data.locked)),
        );''', '''        const ids = session.selection.value.nodeIds;
        const items = session.document.value?.structure.items.filter((item) => ids.includes(item.id)) ?? [];
        await store.setLocked(ids, !items.every((item) => item.locked));''')
write(path, source)
# Remove the Section renderer from ordinary content; the dedicated container renderer is authoritative.
path = "src/features/spaces/presentation/canvas/nodes/SpaceCanvasNodeContent.vue"
source = read(path).replace('import { spaceAssetSource } from "../../../assets/node-preview-source";\n', '')
source = source.replace('const isSection = computed(() => props.node.data.kind === "section");\n', '')
start = source.index("const sectionBackground = computed(")
end = source.index("const contentDetailLevel", start)
source = source[:start] + source[end:]
source = source.replace("      isSection\n        ? 'rounded-canvas-object border border-transparent bg-transparent'\n        : isText", "      isText")
source = source.replace(':style="[sectionBackground, (isNote || isText) && textPresentation.style]"', ':style="(isNote || isText) ? textPresentation.style : undefined"')
source = re.sub(r'\s*<span v-if="node.data.locked"[^>]*>Locked</span>', '', source)
write(path, source)
for path in ("src/features/spaces/presentation/canvas/nodes/SpaceCanvasNote.vue", "src/features/spaces/presentation/canvas/nodes/SpaceCanvasText.vue"):
    source = read(path).replace(" && !data.locked", "").replace(" && !props.data.locked", "")
    write(path, source)
path = "src/features/spaces/presentation/canvas/nodes/SpaceCanvasWebsite.vue"
source = read(path)
source = re.sub(r'\s*<LockKeyhole v-if="data.locked"[^>]*/>', '', source)
source = source.replace("LockKeyhole, ", "").replace(", LockKeyhole", "")
write(path, source)
path = "src/features/spaces/presentation/canvas/nodes/use-canvas-text-editor.ts"
source = read(path).replace(" || options.data.value.locked", "")
match = re.search(r'  watch\(\n    \(\) => options\.data\.value\.locked,[\s\S]*?\n  \);\n', source)
if match: source = source[:match.start()] + source[match.end():]
write(path, source)
path = "src/features/spaces/presentation/canvas/nodes/space-canvas-node-content.ts"
write(path, read(path).replace('    case "section":\n', ''))
for path in ("src/features/spaces/presentation/node-icon.ts", "src/features/spaces/presentation/canvas/nodes/space-canvas-node-presentation.ts"):
    write(path, re.sub(r'^  section:.*\n', '', read(path), flags=re.M))
# Architectural projections, not payload fields, expose effective edit protection to the inspector.
write("src/features/spaces/presentation/use-selected-structure.ts", r'''
import { computed } from "vue";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { indexStructure, structureAncestors } from "../model/structure/structure-index";

export function useSelectedStructure() {
  const session = useCanvasEditorSession();
  const structure = computed(() => session.document.value?.structure);
  const selected = computed(() => {
    const [id, second] = session.selection.value.nodeIds;
    if (!structure.value || !id || second) return undefined;
    const tree = indexStructure(structure.value);
    const item = tree.items.get(id);
    return item ? { item, locked: item.locked || structureAncestors(tree, id).some((parent) => parent.locked) } : undefined;
  });
  return { selected, session };
}
''')
path = "src/features/spaces/presentation/SpaceNodeIdentity.vue"
source = read(path)
source = source.replace('<script setup lang="ts">', '<script setup lang="ts">\nimport { useSelectedStructure } from "./use-selected-structure";')
source = source.replace('const kindIcon =', 'const { selected: structureSelection } = useSelectedStructure();\nconst kindIcon =')
source = source.replace('v-if="node.data.locked"', 'v-if="structureSelection?.locked"')
write(path, source)
# Existing negative Section checks on actual content are removed, not weakened with casts.
for path in ("src/features/spaces/presentation/canvas/SpaceCanvasProductLayer.vue", "src/features/spaces/search/use-find.ts", "src/features/spaces/package/restore.ts"):
    source = read(path).replace('.filter((node) => node.data.kind !== "section")', '').replace('.filter((node) => node.kind !== "section")', '')
    source = source.replace('node.data.kind !== "section" && ', '')
    write(path, source)
path = "src/features/spaces/canvas/konva/interaction/use-node-content-activation.ts"
source = read(path).replace('    if (!item) return;', '    if (!item || !item.interaction.movable) return;')
source = source.replace('if (selected?.interaction.activationMode === "select-to-interact")', 'if (selected?.interaction.movable && selected.interaction.activationMode === "select-to-interact")')
source = source.replace('return interactiveNodeId.value === item.node.id;', 'return item.interaction.movable && interactiveNodeId.value === item.node.id;')
write(path, source)
# Correct a renderer-selection identity mismatch; selection stores relation IDs, not render keys.
replace("src/features/spaces/canvas/konva/presentation/CanvasRelationsLayer.vue", "selectedRelationIds.has(relation.key)", "selectedRelationIds.has(relation.id)")
replace("src/features/spaces/canvas/konva/presentation/CanvasRelationShape.vue", "      points: relation.points,", "      relationId: relation.id,\n      points: relation.points,")
# Camera changes must not retraverse knowledge scopes.
replace("src/features/spaces/knowledge/assistant/assistant-model.ts", "  const structuralScope = computed(() => {\n    const document = session.document.value;\n    if (!document)", "  const structure = computed(() => session.document.value?.structure);\n  const structuralScope = computed(() => {\n    const documentStructure = structure.value;\n    if (!documentStructure)")
replace("src/features/spaces/knowledge/assistant/assistant-model.ts", "      document.structure,", "      documentStructure,")
# Native portability and visibility errors identified by the exact-source compiler run.
replace("src-tauri/src/spaces/canvas_state/structure_validation.rs", "pub(super) fn valid_id", "pub(in crate::spaces) fn valid_id")
replace("src-tauri/src/ai/providers/agents/discovery.rs", "let applications = std::collections::HashSet::new();", "let applications = std::collections::HashSet::<crate::ai::contracts::AgentKind>::new();")
replace("src-tauri/src/desktop/mod.rs", "use tauri::Emitter;", "use tauri::{Emitter, Manager};")
# Replace obsolete flat-document tests with the current storage contract and transaction coverage.
write("src-tauri/src/spaces/canvas_state/tests/mod.rs", r'''
use super::{contract::SpaceEditorDocument, validate_document};
use sqlx::sqlite::SqlitePoolOptions;

fn content_document() -> SpaceEditorDocument {
    serde_json::from_value(serde_json::json!({
        "version": 1, "viewport": {"x":0,"y":0,"zoom":1},
        "structure": {"version":1,"items":[{"id":"node","nodeId":"node","kind":"content","parentId":null,"order":0,"locked":false}]},
        "layout": {"version":1,"items":[{"kind":"content","itemId":"node","transform":{"x":1,"y":2,"rotation":0,"scale":1},"size":{"width":320,"height":180},"sizeAuthority":"frame"}]}
    })).expect("current document")
}
#[test]
fn validates_separate_structure_and_local_layout() { validate_document(&content_document()).expect("valid structure document"); }
#[test]
fn rejects_legacy_flat_documents() {
    assert!(serde_json::from_str::<SpaceEditorDocument>(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#).is_err());
}
#[test]
fn rejects_engine_fields_and_duplicate_layouts() {
    let mut value = serde_json::to_value(content_document()).expect("serialize");
    value["store"] = serde_json::json!({});
    assert!(serde_json::from_value::<SpaceEditorDocument>(value).is_err());
    let mut document = content_document(); document.layout.items.push(document.layout.items[0].clone());
    assert!(validate_document(&document).is_err());
}
#[test]
fn relation_colors_are_non_default_and_have_unique_identities() {
    let mut value = serde_json::to_value(SpaceEditorDocument::empty()).expect("serialize");
    value["relationStyles"] = serde_json::json!([{"relationId":"r","color":"blue"}]);
    validate_document(&serde_json::from_value(value.clone()).expect("style")).expect("valid color");
    value["relationStyles"][0]["color"] = serde_json::json!("neutral");
    assert!(serde_json::from_value::<SpaceEditorDocument>(value).is_err());
}
async fn database() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("sqlite");
    crate::spaces::db::migrate(&pool).await.expect("migrations");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES('space','space','Space',1,1,1)").execute(&pool).await.expect("space");
    pool
}
#[tokio::test]
async fn stale_revision_rolls_back_content_and_structure_together() {
    let pool = database().await;
    super::repository::save(&pool,"space",0,&SpaceEditorDocument::empty()).await.expect("initial state");
    let mut tx = pool.begin().await.expect("transaction");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Node\",\"tiptapContent\":{\"type\":\"doc\",\"content\":[]},\"plainText\":\"\"}',1,1)")
        .execute(&mut *tx).await.expect("staged content");
    let error = super::repository::save_in_transaction(&mut tx,"space",0,&content_document()).await.expect_err("stale revision");
    assert!(error.contains("revision conflict"));
    tx.rollback().await.expect("rollback");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes").fetch_one(&pool).await.expect("count");
    assert_eq!(count,0);
    let (document,revision) = super::repository::load(&pool,"space").await.expect("load").expect("state");
    assert_eq!(revision,1); assert!(document.structure.items.is_empty());
}
#[tokio::test]
async fn camera_persistence_does_not_advance_the_edit_revision() {
    let pool = database().await;
    super::repository::save(&pool,"space",0,&SpaceEditorDocument::empty()).await.expect("initial state");
    let mut tx = pool.begin().await.expect("transaction");
    super::viewport::save_in_transaction(&mut tx,"space",&super::contract::CanvasViewport { x:120.0,y:-40.0,zoom:0.8 }).await.expect("camera");
    tx.commit().await.expect("commit");
    let (document,revision) = super::repository::load(&pool,"space").await.expect("load").expect("state");
    assert_eq!(revision,1); assert_eq!(document.viewport.x,120.0); assert_eq!(document.viewport.zoom,0.8);
}
''')
for path, content in sources.items():
    destination = root / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(content)
marker.parent.mkdir(parents=True, exist_ok=True)
marker.write_text("Reviewed interface integration applied. Remove development markers before delivery.\n")
print(f"Applied interface integration to {len(sources)} source files")
