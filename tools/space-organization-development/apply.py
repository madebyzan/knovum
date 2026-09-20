"""Reviewed Section/Group product presentation and drawing integration."""
from pathlib import Path
import re
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
write('src/features/spaces/canvas/konva/presentation/CanvasNodeShape.vue', '''
<script setup lang="ts">
import type Konva from "konva";
import { computed } from "vue";
import { Group as KonvaGroup, Rect as KonvaRect } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";
import type { CanvasSceneNode } from "../scene/scene-projection";
import type { CanvasPalette } from "./use-canvas-palette";
import { SECTION_HEADER_SCREEN_HEIGHT } from "../../contract/section-metrics";
const props = defineProps<{ interactive: boolean; item: CanvasSceneNode; palette: CanvasPalette; zoom: number; selected: boolean }>();
const emit = defineEmits<{
  activate: [id: string]; dragEnd: [id: string]; dragStart: [id: string];
  dragMove: [id: string, event: Konva.KonvaEventObject<DragEvent>];
  pointerDown: [id: string, event: Konva.KonvaEventObject<PointerEvent>];
}>();
const frame = computed(() => props.item.layout.frame);
const section = computed(() => props.item.node.data.kind === "section");
const group = computed(() => props.item.node.data.kind === "group");
const collapsed = computed(() => props.item.node.data.kind === "section" && props.item.node.data.collapsed);
const color = computed(() => {
  const appearance = props.item.layout.appearance;
  return appearance?.customFill ?? (appearance?.fill ? props.palette.noteFill[appearance.fill] : props.palette.surface);
});
const config = computed(() => ({
  id: `canvas-node-${props.item.node.id}`, name: "canvas-node", nodeId: props.item.node.id,
  x: frame.value.x, y: frame.value.y, rotation: frame.value.rotation,
  draggable: props.interactive && props.item.interaction.movable,
}));
</script>
<template>
  <KonvaGroup :config="config" @dblclick="interactive && emit('activate', item.node.id)"
    @dragstart="emit('dragStart', item.node.id)" @dragend="emit('dragEnd', item.node.id)"
    @dragmove="emit('dragMove', item.node.id, $event)" @pointerdown="emit('pointerDown', item.node.id, $event)">
    <KonvaRect :config="{ width: frame.width, height: frame.height, fill: section ? color : 'transparent',
      opacity: section ? 0.12 : 1, listening: section ? collapsed : !group || selected,
      strokeEnabled: false, cornerRadius: section ? 6 : 0, perfectDrawEnabled: false }" />
    <KonvaRect v-if="section" :config="{ width: frame.width, height: frame.height, listening: false,
      stroke: selected ? palette.accent : palette.border, strokeWidth: 1 / zoom,
      strokeScaleEnabled: false, cornerRadius: 6, perfectDrawEnabled: false }" />
    <KonvaRect v-if="section && !collapsed" :config="{ width: frame.width,
      height: Math.min(frame.height, SECTION_HEADER_SCREEN_HEIGHT / zoom), fill: 'transparent', perfectDrawEnabled: false }" />
  </KonvaGroup>
</template>
''')
write('src/features/spaces/canvas/konva/presentation/CanvasContainerTitle.vue', '''
<script setup lang="ts">
import { EditableArea, EditableInput, EditablePreview, EditableRoot } from "reka-ui";
import { nextTick, ref, useId, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
const props = defineProps<{ title: string; editing: boolean; save(title: string): Promise<void> }>();
const emit = defineEmits<{ finish: [] }>();
const { t } = useI18n();
const field = useTemplateRef<InstanceType<typeof EditableRoot>>("field");
const draft = ref(props.title), saving = ref(false), error = ref("");
const errorId = useId();
watch(() => props.editing, async (active) => {
  if (!active) return;
  draft.value = props.title; error.value = "";
  await nextTick(); field.value?.edit();
}, { immediate: true });
async function save() {
  if (saving.value) return;
  const title = draft.value.trim();
  if (!title || title.length > 512) { error.value = t("spaces.structure.renameInvalid"); return; }
  saving.value = true; error.value = "";
  try { await props.save(title); field.value?.submit(); emit("finish"); }
  catch (cause) { error.value = `${t("spaces.structure.saveFailed")} ${String(cause)}`; }
  finally { saving.value = false; }
}
function key(event: KeyboardEvent) {
  event.stopPropagation();
  if (event.isComposing || saving.value) return;
  if (event.key === "Enter") { event.preventDefault(); void save(); }
}
function blur() { if (!saving.value && !error.value) field.value?.cancel(); }
</script>
<template>
  <EditableRoot v-if="editing" ref="field" v-model="draft" activation-mode="none" submit-mode="none" select-on-focus
    :readonly="saving" :max-length="512" class="pointer-events-auto min-w-0 flex-1" @pointerdown.stop @dblclick.stop
    @update:state="(state) => state === 'cancel' && emit('finish')">
    <EditableArea class="w-full">
      <EditablePreview class="truncate text-label font-medium" />
      <EditableInput :aria-label="t('spaces.structure.rename')" :aria-invalid="Boolean(error)" :aria-describedby="error ? errorId : undefined"
        class="h-7 w-full min-w-0 rounded-control border border-accent bg-panel px-1 text-label text-foreground outline-none"
        @keydown="key" @blur="blur" />
    </EditableArea>
    <p v-if="error" :id="errorId" role="alert" class="absolute top-full left-0 mt-1 max-w-sm rounded-control border border-border bg-overlay p-2 text-caption text-danger">{{ error }}</p>
  </EditableRoot>
  <strong v-else class="min-w-0 flex-1 truncate text-label font-medium text-foreground" :title="title">{{ title }}</strong>
</template>
''')
write('src/features/spaces/canvas/konva/presentation/CanvasContainerMenu.vue', '''
<script setup lang="ts">
import { Ellipsis } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { CanvasEditorPort } from "../../contract/canvas-editor-port";
const props = defineProps<{ id: string; kind: "group" | "section"; title: string; locked: boolean; ancestorLocked: boolean; count: number; editor: CanvasEditorPort; reportError(error: unknown): void }>();
const { t } = useI18n();
const run = (work: () => Promise<unknown>) => { void work().catch(props.reportError); };
</script>
<template>
  <span class="pointer-events-auto" @pointerdown.stop @dblclick.stop>
    <UiDropdownMenu align="start" side="bottom">
      <template #trigger><UiIconButton :label="t('spaces.structure.organize')" size="sm"><Ellipsis class="size-(--knovum-icon-size-sm)" /></UiIconButton></template>
      <UiDropdownMenuItem :disabled="locked || ancestorLocked" @select="editor.requestContainerRename(id)">{{ t('spaces.structure.rename') }}</UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="kind === 'group'" :disabled="locked || ancestorLocked" @select="editor.enterGroup(id)">{{ t('spaces.structure.enterGroup') }}</UiDropdownMenuItem>
      <UiDropdownMenuItem v-else :disabled="locked || ancestorLocked || !count" @select="run(() => editor.fitSection(id))">{{ t('spaces.structure.fit') }}</UiDropdownMenuItem>
      <UiDropdownMenuItem :disabled="ancestorLocked" @select="run(() => editor.executeStructureCommand({ kind: 'set-items-locked', ids: [id], locked: !locked }))">{{ t(locked ? 'spaces.structure.unlock' : 'spaces.structure.lock') }}</UiDropdownMenuItem>
      <UiSeparator class="my-1" />
      <UiDropdownMenuItem :disabled="locked || ancestorLocked" @select="run(() => editor.executeStructureCommand({ kind: 'dissolve-container', id }))">{{ t(kind === 'group' ? 'spaces.structure.ungroup' : 'spaces.structure.dissolveSection') }}</UiDropdownMenuItem>
      <UiDropdownMenuItem :disabled="locked || ancestorLocked" @select="run(() => editor.deleteItems([id]))">{{ t(count ? 'spaces.structure.deleteContainer' : 'spaces.structure.deleteEmpty', { title, count }) }}</UiDropdownMenuItem>
    </UiDropdownMenu>
  </span>
</template>
''')
write('src/features/spaces/canvas/konva/presentation/container-chrome-projection.ts', '''
import { indexStructure, structureAncestors } from "../../../model/structure/structure-index";
import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasViewport } from "../../contract/canvas-editor-port";
import type { CanvasScene } from "../scene/canvas-scene";
export function containerChromeProjection(document: SpaceEditorDocument, scene: CanvasScene, previews: ReadonlyMap<string, CanvasNodeLayoutV1>, viewport: CanvasViewport, visible: ReadonlySet<string>, selected: ReadonlySet<string>, renameId: string | undefined) {
  const tree = indexStructure(document.structure);
  const layouts = new Map(document.layout.items.map((item) => [item.itemId, item]));
  return scene.nodes.flatMap((node) => {
    const item = tree.items.get(node.node.id)!;
    if (item.kind === "content" || !visible.has(item.id)) return [];
    if (item.kind === "group" && !selected.has(item.id) && renameId !== item.id) return [];
    const layout = layouts.get(item.id)!;
    const frame = (previews.get(item.id) ?? node.layout).frame;
    const count = node.node.data.kind === "group" || node.node.data.kind === "section" ? node.node.data.contentCount : 0;
    return [{ item, frame, count, ancestorLocked: structureAncestors(tree, item.id).some((parent) => parent.locked),
      collapsed: layout.kind === "section" && layout.collapsed,
      style: { left: `${viewport.x + frame.x * viewport.zoom + 8}px`, top: `${viewport.y + frame.y * viewport.zoom + (item.kind === "group" ? -32 : 4)}px`, width: `${Math.max(96, frame.width * viewport.zoom - 16)}px` },
    }];
  });
}
''')
write('src/features/spaces/canvas/konva/presentation/CanvasContainerChrome.vue', '''
<script setup lang="ts">
import { ChevronDown, ChevronRight, Frame, Group, Lock } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { CanvasEditorPort, CanvasViewport } from "../../contract/canvas-editor-port";
import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasScene } from "../scene/canvas-scene";
import { containerChromeProjection } from "./container-chrome-projection";
import CanvasContainerTitle from "./CanvasContainerTitle.vue";
import CanvasContainerMenu from "./CanvasContainerMenu.vue";
const props = defineProps<{
  document: SpaceEditorDocument; scene: CanvasScene; viewport: CanvasViewport;
  previews: ReadonlyMap<string, CanvasNodeLayoutV1>; selected: ReadonlySet<string>; visible: ReadonlySet<string>;
  renameId: string | undefined; editor: CanvasEditorPort; dropTarget: string | null | undefined;
  interactive: boolean; reportError(error: unknown): void;
}>();
const emit = defineEmits<{ finishRename: [] }>();
const { t } = useI18n();
const containers = computed(() => containerChromeProjection(props.document, props.scene, props.previews, props.viewport, props.visible, props.selected, props.renameId));
function collapse(id: string, collapsed: boolean) { void props.editor.executeStructureCommand({ kind: "set-section-collapsed", id, collapsed }).catch(props.reportError); }
</script>
<template>
  <div class="pointer-events-none absolute inset-0 z-20 overflow-hidden" data-thumbnail-exclude>
    <div v-for="container in containers" :key="container.item.id" :style="container.style" :data-structure-id="container.item.id"
      class="absolute flex h-7 items-center gap-1 rounded-control" :class="selected.has(container.item.id) && 'bg-overlay/90'">
      <button v-if="container.item.kind === 'section'" type="button" class="pointer-events-auto grid size-6 shrink-0 place-items-center rounded-control text-foreground-secondary hover:bg-control-hover"
        :aria-label="t(container.collapsed ? 'spaces.structure.expand' : 'spaces.structure.collapse')" :aria-expanded="!container.collapsed" :disabled="!interactive"
        @pointerdown.stop @dblclick.stop @click.stop="collapse(container.item.id, !container.collapsed)">
        <component :is="container.collapsed ? ChevronRight : ChevronDown" class="size-(--knovum-icon-size-xs)" />
      </button>
      <component :is="container.item.kind === 'section' ? Frame : Group" class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
      <CanvasContainerTitle :title="container.item.title" :editing="renameId === container.item.id && interactive"
        :save="(title) => editor.executeStructureCommand({ kind: 'rename-container', id: container.item.id, title })" @finish="emit('finishRename')" />
      <span v-if="viewport.zoom > .4" class="shrink-0 text-micro text-foreground-tertiary">{{ t('spaces.structure.items', { count: container.count }) }}</span>
      <Lock v-if="container.item.locked || container.ancestorLocked" class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
      <CanvasContainerMenu v-if="interactive && selected.has(container.item.id)" :id="container.item.id" :kind="container.item.kind" :title="container.item.title" :count="container.count"
        :locked="container.item.locked" :ancestor-locked="container.ancestorLocked" :editor="editor" :report-error="reportError" />
    </div>
    <div v-if="dropTarget" role="status" class="absolute bottom-12 left-1/2 -translate-x-1/2 rounded-control border border-accent bg-overlay px-3 py-1 text-caption text-accent">
      {{ t('spaces.structure.dropInto', { title: containers.find((item) => item.item.id === dropTarget)?.item.title ?? '' }) }}
    </div>
  </div>
</template>
''')
write('src/features/spaces/canvas/konva/interaction/use-section-drawing.ts', '''
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasPoint } from "../../contract/canvas-editor-port";
import { SECTION_DEFAULT_SIZE, SECTION_MINIMUM_SIZE } from "../../contract/section-metrics";
import type { StructureCommand } from "../../../organization/structure/commands/contract";
import { indexStructureSnapshot } from "../../../organization/structure/structure-context";
import { projectStructure } from "../../../organization/structure/structure-projection";
export function useSectionDrawing(options: { document(): SpaceEditorDocument; zoom(): number; exitScope(): void; execute(command: StructureCommand): Promise<void>; complete(): void; reportError(error: unknown): void }) {
  const { t } = useI18n();
  const draft = shallowRef<{ start: CanvasPoint; end: CanvasPoint }>();
  const pending = shallowRef(false);
  const click = computed(() => !draft.value || Math.hypot(draft.value.end.x - draft.value.start.x, draft.value.end.y - draft.value.start.y) * options.zoom() < 5);
  const bounds = computed(() => {
    const value = draft.value; if (!value) return undefined;
    if (click.value) return { ...value.start, ...SECTION_DEFAULT_SIZE };
    const dx = value.end.x - value.start.x, dy = value.end.y - value.start.y;
    const width = Math.max(SECTION_MINIMUM_SIZE.width, Math.abs(dx)), height = Math.max(SECTION_MINIMUM_SIZE.height, Math.abs(dy));
    return { x: dx < 0 ? value.start.x - width : value.start.x, y: dy < 0 ? value.start.y - height : value.start.y, width, height };
  });
  const candidates = computed(() => {
    const region = bounds.value; if (!region || click.value) return [];
    const context = indexStructureSnapshot(options.document());
    const projection = projectStructure(context);
    return (context.tree.children.get(null) ?? []).filter((item) => {
      if (item.kind === "section" || item.locked) return false;
      const rect = projection.bounds.get(item.id)!;
      return rect.x >= region.x && rect.y >= region.y && rect.x + rect.width <= region.x + region.width && rect.y + rect.height <= region.y + region.height;
    }).map((item) => item.id);
  });
  function begin(point: CanvasPoint) { if (!pending.value) { options.exitScope(); draft.value = { start: point, end: point }; } }
  function move(point: CanvasPoint) { if (draft.value) draft.value = { ...draft.value, end: point }; }
  function cancel() { draft.value = undefined; }
  async function finish() {
    const region = bounds.value; if (!region || pending.value) return;
    const ids = candidates.value; cancel(); pending.value = true;
    try { await options.execute({ kind: "draw-section", id: crypto.randomUUID(), title: t("spaces.structure.untitledSection"), bounds: region, ids }); options.complete(); }
    catch (error) { options.reportError(error); }
    finally { pending.value = false; }
  }
  return { bounds, candidates, pending, begin, move, cancel, finish };
}
''')
write('src/features/spaces/canvas/konva/presentation/CanvasStructureFeedback.vue', '''
<script setup lang="ts">
import { computed } from "vue";
import { Group, ArrowLeft } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import type { CanvasViewport } from "../../contract/canvas-editor-port";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
import type { StructureBounds } from "../../contract/structure-layout";
const props = defineProps<{ document: SpaceEditorDocument; groupId: string | null; viewport: CanvasViewport; drawing: boolean; bounds: StructureBounds | undefined; candidates: readonly string[]; dropTarget: string | null | undefined }>();
const emit = defineEmits<{ exit: [] }>();
const { t } = useI18n();
const title = computed(() => props.document.structure.items.find((item) => item.id === props.groupId));
function style(bounds: StructureBounds) {
  return { left: `${props.viewport.x + bounds.x * props.viewport.zoom}px`, top: `${props.viewport.y + bounds.y * props.viewport.zoom}px`, width: `${bounds.width * props.viewport.zoom}px`, height: `${bounds.height * props.viewport.zoom}px` };
}
const dropStyle = computed(() => {
  const layout = props.document.layout.items.find((item) => item.itemId === props.dropTarget);
  return layout?.kind === "section" ? style({ ...layout.position, ...(layout.collapsed ? { width: 320, height: 64 } : layout.size) }) : undefined;
});
</script>
<template>
  <div class="pointer-events-none absolute inset-0 z-20 overflow-hidden" data-thumbnail-exclude>
    <div v-if="bounds" class="absolute rounded-control border-2 border-dashed border-accent bg-accent/5" :style="style(bounds)" />
    <div v-if="dropStyle" class="absolute rounded-control border-2 border-accent bg-accent/5" :style="dropStyle" />
    <div v-if="groupId && title?.kind === 'group'" class="pointer-events-auto absolute top-3 left-3 flex h-8 items-center gap-2 rounded-control border border-border bg-overlay px-2 shadow-toolbar" data-testid="group-edit-scope">
      <button type="button" :aria-label="t('spaces.structure.exitGroup')" class="rounded-control p-1 hover:bg-control-hover" @click="emit('exit')"><ArrowLeft class="size-(--knovum-icon-size-sm)" /></button>
      <Group class="size-(--knovum-icon-size-sm) text-foreground-secondary" /><span class="text-label">{{ title.title }}</span><span class="text-micro text-foreground-tertiary">Esc</span>
    </div>
    <div v-if="drawing" role="status" class="absolute bottom-12 left-1/2 -translate-x-1/2 rounded-control border border-border bg-overlay px-3 py-1 text-caption text-foreground-secondary">
      {{ bounds ? t('spaces.structure.drawCaptured', { count: candidates.length }) : t('spaces.structure.drawing') }}
    </div>
  </div>
</template>
''')
name='src/features/spaces/canvas/konva/presentation/use-canvas-scene-layer-projection.ts';text=read(name)
text=text.replace('    visibleContentNodes:', '    visibleGroups: computed(() => props.scene.groups.filter(({ node }) => props.visibleNodeIds.has(node.id))),\n    visibleContentNodes:')
write(name,text)
name='src/features/spaces/canvas/konva/presentation/CanvasSceneLayers.vue';text=read(name)
text=text.replace('  visibleContentNodes,','  visibleContentNodes,\n  visibleGroups,')
text=text.replace('      :item="item"','      :item="item"\n      :selected="selectedNodeIds.has(item.node.id)"')
needle='    <CanvasNodeShape\n      :interactive="interactive"'
text=text.replace(needle,'''    <CanvasNodeShape v-for="item in visibleGroups" :key="item.node.id" :interactive="interactive"
      :item="item" :selected="selectedNodeIds.has(item.node.id)" :palette="palette" :zoom="zoom"
      @activate="emit('activate', $event)" @drag-end="emit('dragEnd', $event)"
      @drag-move="(id, event) => emit('dragMove', id, event)" @drag-start="emit('dragStart', $event)" @pointer-down="relayPointerDown" />
'''+needle)
write(name,text)
name='src/features/spaces/canvas/contract/canvas-tool.ts';write(name,read(name).replace(' | "text"', ' | "text" | "section"'))
name='src/features/spaces/commands/canvas-tool-commands.ts';text=read(name).replace('] as const', '  { tool: "section", title: "spaces.structure.drawSection", shortcut: "S" },\n] as const')
write(name,text)
name='src/features/spaces/presentation/canvas/SpaceCanvasToolbar.vue';text=read(name).replace('Hand, MousePointer2, StickyNote, Type','Hand, MousePointer2, StickyNote, Type, Frame').replace('text: Type }','text: Type, section: Frame }')
write(name,text)
name='src/features/spaces/canvas/konva/interaction/use-pointer-interactions.ts';text=read(name)
text=text.replace('  resolveHit(id: string): string;', '  resolveHit(id: string): string;\n  sectionDrawing: { begin(point: CanvasPoint): void; move(point: CanvasPoint): void; finish(): Promise<void>; cancel(): void };')
text=text.replace('    if (options.viewport.panning.value) options.viewport.movePan(event);','    if (options.tool.value === "section" && lastCanvasPoint.value && !options.viewport.panning.value) options.sectionDrawing.move(lastCanvasPoint.value);\n    else if (options.viewport.panning.value) options.viewport.movePan(event);')
text=text.replace('    else options.selection.endMarquee();','    else if (options.tool.value === "section") void options.sectionDrawing.finish();\n    else options.selection.endMarquee();')
text=text.replace('    options.cancelActiveGesture();','    options.cancelActiveGesture();\n    options.sectionDrawing.cancel();')
text=text.replace('    const point = lastCanvasPoint.value;\n    if (point','    const point = lastCanvasPoint.value;\n    if (tool === "section") { if (point) options.sectionDrawing.begin(point); return true; }\n    if (point')
write(name,text)
name='src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts';text=read(name)
text='import { useSectionDrawing } from "./interaction/use-section-drawing";\n'+text
text=text.replace('  const pointer = usePointerInteractions({','''  const drawing = useSectionDrawing({ document: () => model.document, zoom: () => viewport.viewport.value.zoom,
    exitScope: () => { scope.groupId.value = null; selection.clear(); }, execute: gestures.runStructureCommand,
    complete: props.events.onStructureToolFinished, reportError: props.events.onError });
  watch(() => props.tool, () => drawing.cancel());
  const pointer = usePointerInteractions({
    sectionDrawing: drawing,''')
text=text.replace('if (scope.renameId.value) scope.renameId.value = undefined;', 'if (drawing.bounds.value) drawing.cancel(); else if (scope.renameId.value) scope.renameId.value = undefined;')
text=text.replace('    editor,','    drawing,\n    editor,')
write(name,text)
name='src/features/spaces/canvas/konva/KonvaCanvasSurface.vue';text=read(name)
text='''TEMP_MARKER'''+text
text=text.replace('TEMP_MARKER<script setup lang="ts">','<script setup lang="ts">\nimport CanvasContainerChrome from "./presentation/CanvasContainerChrome.vue";\nimport CanvasStructureFeedback from "./presentation/CanvasStructureFeedback.vue";')
text=text.replace("tool === 'note' || tool === 'text'", "tool === 'note' || tool === 'text' || tool === 'section'")
text=text.replace("tool === 'select' &&\n          !runtime.viewport", "tool === 'select' &&\n          !runtime.gestures.busy.value &&\n          !runtime.viewport")
text=text.replace('    <Teleport v-if="runtime.stage.value"', '''    <CanvasStructureFeedback :document="runtime.model.document" :group-id="runtime.scope.groupId.value" :viewport="runtime.viewport.viewport.value"
      :drawing="tool === 'section'" :bounds="runtime.drawing.bounds.value" :candidates="runtime.drawing.candidates.value" :drop-target="runtime.gestures.dropTarget.value" @exit="runtime.scope.exit" />
    <CanvasContainerChrome v-if="runtime.editor.value" :document="runtime.model.document" :scene="runtime.scene.value" :viewport="runtime.viewport.viewport.value"
      :previews="runtime.scenePreview.layouts" :selected="runtime.selection.nodeIds.value" :visible="runtime.visibility.visibleNodeIds.value" :rename-id="runtime.scope.renameId.value"
      :editor="runtime.editor.value" :drop-target="runtime.gestures.dropTarget.value" :interactive="interactionEnabled && tool === 'select' && !runtime.gestures.busy.value"
      :report-error="events.onError" @finish-rename="runtime.scope.renameId.value = undefined" />
    <Teleport v-if="runtime.stage.value"''')
write(name,text)
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Integrated Section regions, drawing and primitive-owned container title editing.')
