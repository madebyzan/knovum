import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { structureSelectionEligibility } from "../organization/structure/structure-eligibility";
export function useStructureCommands() {
  const session = useCanvasEditorSession();
  const { t } = useI18n();
  const eligibility = () =>
    session.document.value
      ? structureSelectionEligibility(session.document.value.structure, session.selection.value.nodeIds)
      : undefined;
  return computed<readonly AppCommand[]>(() => [
    {
      id: "spaces.group-selection",
      title: t("spaces.structure.groupSelection"),
      shortcut: "$mod+G",
      category: "editing",
      isEnabled: () => eligibility()?.canGroup === true,
      run: () =>
        session.activeEditor.value?.executeStructureCommand({
          kind: "group-selection",
          id: crypto.randomUUID(),
          title: t("spaces.structure.untitledGroup"),
          ids: [...session.selection.value.nodeIds],
        }),
    },
    {
      id: "spaces.ungroup-selection",
      title: t("spaces.structure.ungroup"),
      shortcut: "$mod+Shift+G",
      category: "editing",
      isEnabled: () => eligibility()?.canUngroup === true,
      run: () => {
        const id = eligibility()?.roots[0]?.id;
        if (id)
          return session.activeEditor.value?.executeStructureCommand({ kind: "dissolve-container", id });
      },
    },
    {
      id: "spaces.section-around-selection",
      title: t("spaces.structure.sectionFromSelection"),
      category: "editing",
      isEnabled: () => eligibility()?.canSection === true,
      run: () =>
        session.activeEditor.value?.executeStructureCommand({
          kind: "section-around-selection",
          id: crypto.randomUUID(),
          title: t("spaces.structure.untitledSection"),
          ids: [...session.selection.value.nodeIds],
        }),
    },
    {
      id: "spaces.enter-group",
      title: t("spaces.structure.enterGroup"),
      category: "canvas",
      isEnabled: () =>
        eligibility()?.roots.length === 1 &&
        eligibility()?.roots[0]?.kind === "group" &&
        eligibility()?.editable === true,
      run: () => {
        const id = eligibility()?.roots[0]?.id;
        if (id) session.activeEditor.value?.enterGroup(id);
      },
    },
    {
      id: "spaces.exit-group",
      title: t("spaces.structure.exitGroup"),
      category: "canvas",
      isEnabled: () => Boolean(session.selection.value.scopeGroupId),
      run: () => session.activeEditor.value?.exitGroup(),
    },
  ]);
}
