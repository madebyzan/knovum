import type { CanvasRect } from "../canvas/contract/canvas-editor-port";
import type { SectionAssignment } from "../canvas/contract/section-membership";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { omit, type SelectionActionContext, updateSelected } from "./selection-mutations";

export function createSectionActions(context: SelectionActionContext) {
  return {
    createSection: (ids: readonly string[], bounds: CanvasRect, title: string) =>
      createSection(context, ids, bounds, title),
    removeSection: (sectionId: string) => removeSection(context, sectionId),
    deleteSectionWithContents: (sectionId: string) => deleteSectionWithContents(context, sectionId),
    assignSections: (assignments: readonly SectionAssignment[]) => assignSections(context, assignments),
    setSectionCollapsed: (id: string, collapsed: boolean) => setSectionCollapsed(context, id, collapsed),
  };
}

async function createSection(
  context: SelectionActionContext,
  ids: readonly string[],
  bounds: CanvasRect,
  title: string,
) {
  const selected = new Set(ids);
  const members = context.nodes.value.filter((node) => selected.has(node.id) && node.data.kind !== "section");
  if (!members.length) return;
  let createdId: string | undefined;
  await context.recordHistoryMutation(async () => {
    const [section] = await context.addItem(
      {
        kind: "section",
        title,
      },
      { x: bounds.x - 32, y: bounds.y - 56 },
      { reveal: false },
    );
    if (!section) return;
    createdId = section.id;
    await assignSections(
      context,
      members.map(({ id }) => ({ id, sectionId: section.id })),
    );
  });
  return context.nodes.value.find(({ id }) => id === createdId);
}

async function removeSection(context: SelectionActionContext, sectionId: string) {
  const members = context.nodes.value.filter((node) => node.data.sectionId === sectionId);
  await context.recordHistoryMutation(async () => {
    await updateSelected(
      context,
      members.map((node) => node.id),
      (data) => omit(data, "sectionId"),
    );
    await context.deleteItems([sectionId]);
  });
}

async function deleteSectionWithContents(context: SelectionActionContext, sectionId: string) {
  const ids = context.nodes.value
    .filter((node) => node.id === sectionId || node.data.sectionId === sectionId)
    .map((node) => node.id);
  await context.deleteItems(ids);
}

async function assignSections(context: SelectionActionContext, assignments: readonly SectionAssignment[]) {
  const target = new Map(assignments.map((item) => [item.id, item.sectionId]));
  const changes = context.nodes.value.flatMap((node) => {
    if (!target.has(node.id) || node.data.kind === "section") return [];
    const sectionId = target.get(node.id);
    if (node.data.sectionId === sectionId) return [];
    const before = validateSpaceNodePayload(node.data);
    return [{ id: node.id, before, after: sectionId ? { ...before, sectionId } : omit(before, "sectionId") }];
  });
  await context.updateManyData(changes);
}

async function setSectionCollapsed(context: SelectionActionContext, id: string, collapsed: boolean) {
  await updateSelected(
    context,
    [id],
    (data) => (collapsed ? { ...data, collapsed: true } : omit(data, "collapsed")),
    { includeSections: true },
  );
}
