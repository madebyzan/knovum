import { useI18n } from "vue-i18n";

import type { CanvasPoint } from "../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { structureSelectionEligibility } from "../organization/structure/structure-eligibility";
import { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem, spaceSubmenu } from "../workspace/space-menu";
import { useCanvasSelectionActions } from "./canvas/use-canvas-selection-actions";

export function useStructureMenu() {
  const session = useCanvasEditorSession();
  const store = useSpaceStore();
  const actions = useCanvasSelectionActions();
  const { t } = useI18n();
  async function show(request: {
    itemId?: string;
    relationId?: string;
    point?: CanvasPoint;
    rename?(id: string): void;
  }) {
    const editor = session.activeEditor.value;
    if (!editor) return;
    if (request.itemId && !session.selection.value.nodeIds.includes(request.itemId))
      editor.selectNodes([request.itemId]);
    if (!request.itemId && !request.relationId) editor.clearSelection();
    const document = editor.getDocument();
    const ids = [...editor.getSelectionSnapshot().nodeIds];
    const state = structureSelectionEligibility(document.structure, ids);
    const item = state.roots.length === 1 ? state.roots[0] : undefined;
    const label = (key: string) => t(`spaces.structure.${key}`);
    const entry = spaceMenuItem;
    const create = entry(label("newSection"), async () => {
      const id = crypto.randomUUID();
      await editor.executeStructureCommand({
        kind: "create-empty-section",
        id,
        title: label("untitledSection"),
        bounds: { ...(request.point ?? store.nextInsertionPoint()), width: 640, height: 400 },
      });
      editor.requestContainerRename(id);
    });
    await popupSpaceContextMenu([
      ids.length
        ? [
            entry(t("common.copy"), () => editor.copySelection()),
            entry(t("common.cut"), () => editor.cutSelection(), { enabled: state.editable }),
            entry(t("spaces.navigation.duplicateSelection"), () => editor.duplicateSelection()),
          ]
        : request.relationId
          ? []
          : [
              create,
              entry(t("common.paste"), async () => {
                await editor.pasteSelection();
              }),
            ],
      ids.length
        ? [
            entry(label("groupSelection"), actions.group, { enabled: state.canGroup }),
            entry(label("ungroup"), actions.ungroup, { enabled: state.canUngroup }),
            entry(label("sectionFromSelection"), actions.createSection, { enabled: state.canSection }),
          ]
        : [],
      item
        ? [
            entry(
              label("rename"),
              () => {
                if (request.rename) request.rename(item.id);
                else if (item.kind !== "content") editor.requestContainerRename(item.id);
                else store.outlineRenameNodeId = item.id;
              },
              { enabled: state.editable },
            ),
            ...(item.kind === "group"
              ? [entry(label("enterGroup"), () => editor.enterGroup(item.id), { enabled: state.editable })]
              : []),
            ...(item.kind === "section"
              ? [
                  entry(label("fit"), () => editor.fitSection(item.id), { enabled: state.editable }),
                  entry(label("dissolveSection"), actions.removeSection, { enabled: state.editable }),
                ]
              : []),
            entry(item.locked ? label("unlock") : label("lock"), () =>
              editor.executeStructureCommand({
                kind: "set-items-locked",
                ids: [item.id],
                locked: !item.locked,
              }),
            ),
          ]
        : [],
      state.editable && state.roots.every((item) => item.kind !== "section")
        ? [
            spaceSubmenu(
              label("moveToSection"),
              document.structure.items
                .filter((item) => item.kind === "section")
                .map((section) =>
                  entry(
                    section.title,
                    () =>
                      editor.executeStructureCommand({ kind: "reparent-items", ids, parentId: section.id }),
                    { enabled: !section.locked },
                  ),
                ),
            ),
            entry(label("moveToRoot"), () =>
              editor.executeStructureCommand({ kind: "reparent-items", ids, parentId: null }),
            ),
          ]
        : [],
      ids.length || request.relationId
        ? [
            entry(label("deleteSelection"), () => editor.deleteSelection(), {
              enabled: !ids.length || state.editable,
            }),
          ]
        : [],
    ]);
  }
  return { show };
}
