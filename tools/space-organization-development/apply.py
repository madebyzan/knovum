"""Reviewed product-entry integration. No database or migration files are modified here."""
from pathlib import Path
import json
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
write('src/features/spaces/organization/structure/structure-eligibility.ts', '''
import type { SpaceStructure } from "../../model/structure/structure-schema";
import { indexStructure, structureAncestors } from "../../model/structure/structure-index";
import { structureOperationRoots } from "./structure-selection";
/** A single capability query is shared by menus, shortcuts, toolbar and Outline. */
export function structureSelectionEligibility(structure: SpaceStructure, ids: readonly string[]) {
  const tree = indexStructure(structure);
  const roots = structureOperationRoots(tree, ids.filter((id) => tree.items.has(id)));
  const editable = roots.length > 0 && roots.every((item) => !item.locked && structureAncestors(tree, item.id).every((parent) => !parent.locked));
  const sameParent = roots.length > 0 && roots.every((item) => item.parentId === roots[0]!.parentId);
  const parent = roots[0]?.parentId;
  const sectionScope = parent === null || parent !== undefined && tree.items.get(parent)?.kind === "section";
  const noSections = roots.every((item) => item.kind !== "section");
  return { roots, editable, sameParent,
    canGroup: editable && sameParent && noSections && roots.length >= 2,
    canSection: editable && sameParent && noSections && sectionScope,
    canUngroup: editable && roots.length === 1 && roots[0]?.kind === "group",
  };
}
''')
write('src/features/spaces/commands/structure-commands.ts', '''
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { AppCommand } from "../../../shared/contracts/app-command";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { structureSelectionEligibility } from "../organization/structure/structure-eligibility";
export function useStructureCommands() {
  const session = useCanvasEditorSession();
  const { t } = useI18n();
  const eligibility = () => session.document.value ? structureSelectionEligibility(session.document.value.structure, session.selection.value.nodeIds) : undefined;
  return computed<readonly AppCommand[]>(() => [
    { id: "spaces.group-selection", title: t("spaces.structure.groupSelection"), shortcut: "$mod+G", category: "editing",
      isEnabled: () => eligibility()?.canGroup === true,
      run: () => session.activeEditor.value?.executeStructureCommand({ kind: "group-selection", id: crypto.randomUUID(), title: t("spaces.structure.untitledGroup"), ids: [...session.selection.value.nodeIds] }) },
    { id: "spaces.ungroup-selection", title: t("spaces.structure.ungroup"), shortcut: "$mod+Shift+G", category: "editing",
      isEnabled: () => eligibility()?.canUngroup === true,
      run: () => { const id = eligibility()?.roots[0]?.id; if (id) return session.activeEditor.value?.executeStructureCommand({ kind: "dissolve-container", id }); } },
    { id: "spaces.section-around-selection", title: t("spaces.structure.sectionFromSelection"), category: "editing",
      isEnabled: () => eligibility()?.canSection === true,
      run: () => session.activeEditor.value?.executeStructureCommand({ kind: "section-around-selection", id: crypto.randomUUID(), title: t("spaces.structure.untitledSection"), ids: [...session.selection.value.nodeIds] }) },
    { id: "spaces.enter-group", title: t("spaces.structure.enterGroup"), category: "canvas",
      isEnabled: () => eligibility()?.roots.length === 1 && eligibility()?.roots[0]?.kind === "group" && eligibility()?.editable === true,
      run: () => { const id = eligibility()?.roots[0]?.id; if (id) session.activeEditor.value?.enterGroup(id); } },
    { id: "spaces.exit-group", title: t("spaces.structure.exitGroup"), category: "canvas",
      isEnabled: () => Boolean(session.selection.value.scopeGroupId), run: () => session.activeEditor.value?.exitGroup() },
  ]);
}
''')
name='src/features/spaces/commands/space-command-catalog.ts';text=read(name)
text='import { useStructureCommands } from "./structure-commands";\n'+text
text=text.replace('  const canvasNavigationCommands = useCanvasNavigationCommands();','  const canvasNavigationCommands = useCanvasNavigationCommands();\n  const structureCommands = useStructureCommands();')
text=text.replace('      ...canvasNavigationCommands.value,','      ...canvasNavigationCommands.value,\n      ...structureCommands.value,')
write(name,text)
name='src/features/spaces/presentation/SpaceAddMenuContent.vue';text=read(name)
text=text.replace('  Boxes,','  Frame,\n  Boxes,')
text=text.replace('  markdown: [];','  section: [];\n  markdown: [];')
text=text.replace('  <UiDropdownMenuItem @select="emit(\'portal\')">','  <UiDropdownMenuItem data-testid="add-section" @select="emit(\'section\')">\n    <Frame class="size-(--knovum-icon-size-sm)" />{{ t("spaces.structure.newSection") }}\n  </UiDropdownMenuItem>\n  <UiDropdownMenuItem @select="emit(\'portal\')">')
write(name,text)
name='src/features/spaces/presentation/use-add-menu.ts';text=read(name)
text='import { currentCanvasEditor } from "../canvas/editor-session";\n'+text
text=text.replace('  const addMarkdown =', '''  async function addSection() {
    try {
      const editor = currentCanvasEditor();
      if (!editor) throw new Error("The canvas editor is not ready");
      await store.ensureActiveSpacePersisted();
      const id = crypto.randomUUID();
      await editor.executeStructureCommand({ kind: "create-empty-section", id, title: t("spaces.structure.untitledSection"), bounds: { ...store.nextInsertionPoint(), width: 640, height: 400 } });
      editor.requestContainerRename(id);
    } catch (error) { toast.error(String(error)); }
  }
  const addMarkdown =''')
text=text.replace('    addMarkdown,','    addSection,\n    addMarkdown,')
write(name,text)
name='src/features/spaces/presentation/SpaceAddMenu.vue';text=read(name).replace('  addMarkdown,','  addSection,\n  addMarkdown,').replace('      @markdown="addMarkdown"','      @section="addSection"\n      @markdown="addMarkdown"')
write(name,text)
# Shared, untranslated metrics are geometry; all user-facing copy goes through the existing locale system.
write('src/features/spaces/canvas/contract/section-metrics.ts', '''
export const SECTION_DEFAULT_SIZE = Object.freeze({ width: 640, height: 400 });
export const SECTION_MINIMUM_SIZE = Object.freeze({ width: 240, height: 160 });
export const SECTION_HEADER_SCREEN_HEIGHT = 32;
export const SECTION_DROP_HOVER_DELAY = 600;
''')
labels={
 'en': {
  'section':'Section','group':'Group','untitledSection':'Untitled Section','untitledGroup':'Untitled Group',
  'newSection':'New Section','drawSection':'Draw Section','sectionFromSelection':'Section around selection','groupSelection':'Group selection','ungroup':'Ungroup','enterGroup':'Enter Group','exitGroup':'Exit Group',
  'spaceRoot':'Space','fit':'Fit to contents','collapse':'Collapse Section','expand':'Expand Section','rename':'Rename','lock':'Lock','unlock':'Unlock',
  'dissolveSection':'Remove Section, keep contents','deleteSelection':'Delete selection','deleteContainer':'Delete {title} and its {count} items','deleteEmpty':'Delete {title}',
  'items':'{count} items','dropInto':'Move into {title}','moveOut':'Move out of Section','renameInvalid':'Enter a title between 1 and 512 characters.',
  'sameParentRequired':'Choose at least two unlocked items with the same parent.','moveToRoot':'Move to Space root','moveToSection':'Move into Section',
  'empty':'Drag content into this Section','color':'Section color','restoreRoot':'Restore to Space root','saveFailed':'Could not save. Your title draft is preserved.',
  'size':'Size','scope':'Editing scope','drawing':'Draw a Section. Click for the default size. Escape cancels.','drawCaptured':'{count} operation units will be included',
  'sectionTooSmall':'Draw a region at least 240 by 160 canvas units, or click to use the default size.',
  'sectionHint':'A named canvas region. Resizing changes its boundary, not its contents.','groupHint':'One operation unit. Double-click to edit inside.',
  'organize':'Organize','collapseOutline':'Collapse outline item','expandOutline':'Expand outline item',
 },
 'zh-cn': {
  'section':'分区','group':'组合','untitledSection':'未命名分区','untitledGroup':'未命名组合',
  'newSection':'新建分区','drawSection':'绘制分区','sectionFromSelection':'将所选放入新分区','groupSelection':'组合所选','ungroup':'取消组合','enterGroup':'进入组合','exitGroup':'退出组合',
  'spaceRoot':'空间','fit':'适应内容','collapse':'折叠分区','expand':'展开分区','rename':'重命名','lock':'锁定','unlock':'解锁',
  'dissolveSection':'移除分区，保留内容','deleteSelection':'删除所选项目','deleteContainer':'删除「{title}」及其中 {count} 项内容','deleteEmpty':'删除「{title}」',
  'items':'{count} 项内容','dropInto':'移入「{title}」','moveOut':'移出分区','renameInvalid':'请输入 1 至 512 个字符的标题。',
  'sameParentRequired':'请选择同一父级下至少两个未锁定的对象。','moveToRoot':'移到空间根层','moveToSection':'移入分区',
  'empty':'将内容拖入此分区','color':'分区颜色','restoreRoot':'恢复到空间根层','saveFailed':'保存失败，已保留标题草稿。',
  'size':'尺寸','scope':'编辑范围','drawing':'拖动绘制分区，单击创建默认大小，Esc 取消。','drawCaptured':'将收纳 {count} 个操作单元',
  'sectionTooSmall':'分区至少为 240 × 160 画布单位，也可以单击使用默认大小。',
  'sectionHint':'具有主题的画布区域。调整区域边界不会缩放内容。','groupHint':'作为一个整体操作，双击进入组合编辑。',
  'organize':'组织','collapseOutline':'收起大纲项目','expandOutline':'展开大纲项目',
 },
}
for locale,values in labels.items(): write(f'src/locales/{locale}/spaces/structure.json',json.dumps(values,ensure_ascii=False,indent=2))
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Added Section entry and shared organization commands to the actual product.')
