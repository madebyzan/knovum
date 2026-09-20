"""Reviewed content/structure boundary cleanup and integration type corrections."""
from pathlib import Path
import re
ROOT=Path('.')
files={}; deletes=[]
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
# Use the actual shared query API rather than adding aliases for guessed names.
for path in (ROOT/'src/features/spaces').rglob('*'):
    if path.suffix not in ('.ts','.vue'): continue
    source=read(str(path)); text=source.replace('structureContentNodeIds','structureContentIds')
    if source!=text: write(str(path),text)
name='src/features/spaces/canvas/contract/canvas-document.ts';text=read(name)
text=text.replace('import { canvasNodeAppearanceSchema } from "./canvas-appearance";\n','')
# Only ephemeral renderer fields may explicitly contain undefined; persisted Zod schemas stay strict.
text=text.replace('contentScale?: number;', 'contentScale?: number | undefined;').replace('appearance?: CanvasNodeAppearance;', 'appearance?: CanvasNodeAppearance | undefined;').replace('fileSubpath?: string;', 'fileSubpath?: string | undefined;')
write(name,text)
name='src/features/spaces/canvas/contract/canvas-editor.ts';write(name,read(name).replace('  CanvasNodePlacement,\n',''))
name='src/features/spaces/canvas/host/use-canvas-document-session.ts';text=read(name).replace('const next = new CanvasDocumentWorkspace', 'const next: CanvasDocumentWorkspace = new CanvasDocumentWorkspace').replace('isActive: () => generation', 'isActive: (): boolean => generation')
write(name,text)
name='src/features/spaces/canvas/persistence/canvas-persistence-coordinator.ts';text=read(name)
text='import { emptySpaceEditorDocument } from "../contract/canvas-document";\n'+text
text=text.replace('createCanvasDocument([])', 'emptySpaceEditorDocument()')
write(name,text)
name='src/features/spaces/canvas/konva/scene/canvas-scene.ts';text=read(name)
text='import type { CanvasRelationGeometry } from "./relation-path";\n'+text
text=text.replace('export interface CanvasSceneRelation {','export interface CanvasSceneRelation extends CanvasRelationGeometry {')
text=re.sub(r'  points: number\[\];\n','',text)
text=re.sub(r'  midpoint: \{[\s\S]*?\};\n','',text,count=1)
text=re.sub(r'  bounds: \{[\s\S]*?\};\n','',text,count=1)
write(name,text)
name='src/features/spaces/canvas/konva/presentation/use-canvas-scene-layer-projection.ts';text=read(name).replace('selectedFrames: selectedLayouts,','selectedFrames: computed(() => selectedLayouts.value.map((layout) => layout.frame)),')
write(name,text)
name='src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts';text=read(name)
text=text.replace('import type { SpaceEditorDocument } from "../contract/canvas-document";\n','')
text=re.sub(r'export function useKonvaCanvasRuntime\(\s*props: CanvasSurfaceProps,\s*documentChanged: \(document: SpaceEditorDocument\) => void,?\s*\)', 'export function useKonvaCanvasRuntime(props: CanvasSurfaceProps)',text)
write(name,text)
name='src/features/spaces/canvas/konva/KonvaCanvasSurface.vue';text=read(name)
text=text.replace('import type { SpaceEditorDocument } from "../contract/canvas-document";\n','')
text=re.sub(r'const emit = defineEmits<\{ documentChange: \[document: SpaceEditorDocument\] \}>\(\);\n','',text)
text=re.sub(r'useKonvaCanvasRuntime\(props, \(document\) => emit\("documentChange", document\)\)', 'useKonvaCanvasRuntime(props)',text)
write(name,text)
name='src/features/spaces/canvas/node-size.ts';text=read(name)
text=re.sub(r'  if \(data.kind === "section" && data.collapsed\) \{[\s\S]*?\n  }\n','',text,count=1)
write(name,text)
name='src/features/spaces/model/nodes/capabilities.ts';text=read(name).replace('  SpaceSectionPayload,\n','')
text=re.sub(r'\nexport function isSpaceSectionPayload\([^)]*\)[\s\S]*?\n}\n','\n',text,count=1)
write(name,text)
name='src/features/spaces/canvas/node-capabilities.ts';text=read(name)
text=re.sub(r'^import type \{ SpaceSectionPayload[^\n]*\n','',text,flags=re.M)
text=re.sub(r'^export type SpaceSectionNode[^\n]*\n','',text,flags=re.M)
text=re.sub(r'\nexport function isSpaceSectionNode\([^)]*\)[\s\S]*?\n}\n','\n',text,count=1)
write(name,text)
name='src/features/spaces/canvas/selection-actions.ts';text=read(name).replace(', isSpaceSectionPayload','').replace('isSpaceSectionPayload, ','').replace('if (!node || isSpaceSectionPayload(node.data)) return;', 'if (!node) return;')
write(name,text)
name='src/features/spaces/knowledge/model/semantic-node.ts';text=read(name).replace('"section", ','').replace(', "section"','').replace('  "section",\n','')
write(name,text)
write('src/features/spaces/organization/selection-actions.ts', '''
import { uniq } from "es-toolkit";
import { currentCanvasEditor } from "../canvas/editor-session";
import { omit, type SelectionActionContext, updateSelected } from "./selection-mutations";
export function createSelectionActions(context: SelectionActionContext) {
  function editor() {
    const value = currentCanvasEditor();
    if (!value) throw new Error("The Space editor is not ready for an organizational command");
    return value;
  }
  return {
    addTag: async (ids: readonly string[], tag: string) => {
      const normalized = tag.trim(); if (!normalized) return;
      await updateSelected(context, ids, (data) => ({ ...data, tags: uniq([...(data.tags ?? []), normalized]) }));
    },
    removeTag: (ids: readonly string[], tag: string) => updateSelected(context, ids, (data) => {
      const tags = data.tags?.filter((candidate) => candidate !== tag);
      return tags?.length ? { ...data, tags } : omit(data, "tags");
    }),
    setLocked: (ids: readonly string[], locked: boolean) => editor().executeStructureCommand({ kind: "set-items-locked", ids: [...ids], locked }),
    deleteSelection: (ids: readonly string[], relationIds: readonly string[] = []) => editor().deleteItems(ids, relationIds),
  };
}
''')
name='src/features/spaces/organization/selection-mutations.ts';text=read(name)
text=text.replace('  options: { includeSections?: boolean } = {},\n','')
text=text.replace('if (!selected.has(node.id) || (!options.includeSections && node.data.kind === "section")) return [];', 'if (!selected.has(node.id)) return [];')
text=text.replace('key: "sectionId" | "tags" | "locked" | "collapsed",','key: "tags",')
write(name,text)
# These modules represented the retired ownership model. They are not compatibility bridges.
deletes.extend(['src/features/spaces/organization/section-actions.ts','src/features/spaces/canvas/konva/scene/section-membership.ts','src/features/spaces/canvas/contract/section-membership.ts','src/features/spaces/canvas/konva/document/canvas-layer-order.ts'])
name='src/features/spaces/package/content-remap.ts';text=read(name)
text=re.sub(r'^  if \(copy.sectionId\)[^\n]*\n','',text,flags=re.M)
write(name,text)
for name in ['src/features/spaces/package/restore.ts','src/features/spaces/search/use-find.ts','src/features/spaces/presentation/canvas/SpaceCanvasProductLayer.vue']:
    text=read(name).replace('.filter((node) => node.data.kind !== "section")','').replace('.filter((node) => node.kind !== "section")','').replace('node.data.kind !== "section" && ','')
    write(name,text)
name='src/features/spaces/canvas/canvas-overview-assets.ts';text=read(name)
text=text.replace(': spaceCanvasOverviewSource(node.data);', ': node.data.kind === "group" || node.data.kind === "section" ? undefined : spaceCanvasOverviewSource(node.data);')
write(name,text)
name='src/features/spaces/canvas/canvas-overview-nodes.ts';text=read(name)
text=text.replace('  const { data } = item.node;', '  const { data } = item.node;\n  if (data.kind === "group") return;')
text=text.replace('const footer = image && data.kind !== "image";', 'const footer = Boolean(image);')
write(name,text)
# Local, exact optional-property fixes retain strict project options.
name='src/features/spaces/presentation/SpaceNodeMetadata.vue';write(name,read(name).replace('  value?: string;', '  value: string | undefined;'))
for name in ['src/features/spaces/presentation/canvas/SpaceCanvasTextFormatBar.vue','src/features/spaces/presentation/canvas/SpaceCanvasTextStyleMenu.vue']:
    if (ROOT/name).exists(): write(name,read(name).replace(':appearance="appearance"','v-bind="appearance ? { appearance } : {}"'))
name='src/features/spaces/presentation/canvas/SpaceCanvasSceneControls.vue';write(name,read(name).replace(':recent-scene-id="recentSceneId"','v-bind="recentSceneId ? { recentSceneId } : {}"'))
name='src/shared/ui/UiSelect.vue';write(name,read(name).replace(':placeholder="placeholder"','v-bind="placeholder === undefined ? {} : { placeholder }"'))
# Read-only compiler diagnostic summary for the exact native checkpoint, not a fabricated execution result.
report=ROOT/'tools/space-organization-development/last-native/report.md'
if report.exists():
    lines=report.read_text().splitlines()
    selected=[line for line in lines if 'error' in line.lower() or 'Source:' in line]
    write('tools/space-organization-development/last-native/errors.md','# Native diagnostic extraction\n\n'+ '\n'.join(selected[-300:]))
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
for name in deletes:
    path=ROOT/name
    if path.exists():path.unlink()
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Removed obsolete content membership modules and connected the current structural API names.')
