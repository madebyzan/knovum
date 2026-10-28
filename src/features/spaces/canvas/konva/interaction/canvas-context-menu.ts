import type Konva from "konva";
import type { Ref } from "vue";

import { getAppCommand, runAppCommand } from "../../../../../foundation/commands/run-app-command";
import { isEditableTarget } from "../../../../../foundation/dom/editable-target";
import { prefersSystemContextMenu } from "../../../../../foundation/dom/text-selection";
import { canRunAction } from "../../../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../../../shared/contracts/app-command";
import { popupSpaceContextMenu, spaceMenuItem } from "../../../workspace/space-menu";
import { currentCanvasEditor } from "../../editor-session";
import type { useCanvasSelection } from "../selection/use-canvas-selection";

type ContextTarget =
  | { kind: "node"; id: string }
  | { kind: "relation"; ids: readonly string[] }
  | { kind: "structure"; id: string };

interface CanvasContextMenuOptions {
  host: Readonly<Ref<HTMLElement | null>>;
  stage: Readonly<Ref<Konva.Stage | undefined>>;
  selection: ReturnType<typeof useCanvasSelection>;
}

export function createCanvasContextMenu(options: CanvasContextMenuOptions) {
  function pointer(event: MouseEvent) {
    if (prefersSystemContextMenu(event.target)) return;
    event.preventDefault();
    options.host.value?.focus({ preventScroll: true });
    const target = domTarget(event.target) ?? konvaTarget(event, options.stage.value);
    selectTarget(target);
    void show({ x: event.clientX, y: event.clientY });
  }

  function keyboard(event: KeyboardEvent): boolean {
    if (event.isComposing || isEditableTarget(event.target) || !isContextMenuKey(event)) {
      return false;
    }
    event.preventDefault();
    event.stopPropagation();
    const host = options.host.value;
    if (!host) return true;
    void show(selectionMenuPosition(host));
    return true;
  }

  function selectTarget(target: ContextTarget | undefined) {
    if (target?.kind === "node") options.selection.selectNodeForContextMenu(target.id);
    else if (target?.kind === "relation") options.selection.selectRelationForContextMenu(target.ids);
    else if (target?.kind === "structure") options.selection.selectStructureNodeForContextMenu(target.id);
  }

  async function show(position: { x: number; y: number }) {
    const hasNodes = options.selection.nodeIds.value.size + options.selection.structureNodeIds.value.size > 0;
    const hasSelection = hasNodes || options.selection.relationIds.value.size > 0;
    const ids = [
      ...(hasNodes ? ["spaces.cut", "spaces.copy", "spaces.duplicate"] : []),
      ...(hasSelection ? ["spaces.delete-selection"] : []),
      "spaces.paste",
      "spaces.select-all",
    ];
    const commands = ids.flatMap((id) => {
      const command = getAppCommand(id);
      return command ? [command] : [];
    });
    await popupSpaceContextMenu(
      commands.map((command) => commandItem(command)),
      { position },
    );
  }

  return { keyboard, pointer };
}

function isContextMenuKey(event: KeyboardEvent) {
  return event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey);
}

function selectionMenuPosition(host: HTMLElement) {
  const rect = host.getBoundingClientRect();
  const editor = currentCanvasEditor();
  const bounds = editor?.getSelectionBounds();
  const anchor =
    bounds && editor
      ? editor.canvasToViewportPoint({ x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2 })
      : undefined;
  return {
    x: rect.left + (anchor?.x ?? rect.width / 2),
    y: rect.top + (anchor?.y ?? rect.height / 2),
  };
}

function commandItem(command: AppCommand) {
  const availability = command.availability?.();
  return spaceMenuItem(command.title, () => runAppCommand(command), {
    enabled: !availability || canRunAction(availability),
    ...(command.shortcut ? { accelerator: command.shortcut } : {}),
  });
}

function domTarget(target: EventTarget | null): ContextTarget | undefined {
  if (!(target instanceof Element)) return undefined;
  const nodeId = target.closest<HTMLElement>("[data-canvas-node-id]")?.dataset.canvasNodeId;
  return nodeId ? { kind: "node", id: nodeId } : undefined;
}

function konvaTarget(event: MouseEvent, stage: Konva.Stage | undefined): ContextTarget | undefined {
  if (!stage) return undefined;
  stage.setPointersPositions(event);
  let target: Konva.Node | null = stage.getIntersection(stage.getPointerPosition() ?? { x: 0, y: 0 });
  while (target && target !== stage) {
    const nodeId = target.getAttr("nodeId") as string | undefined;
    if (nodeId) return { kind: "node", id: nodeId };
    const relationIds = target.getAttr("relationIds") as readonly string[] | undefined;
    if (relationIds) return { kind: "relation", ids: relationIds };
    const structureId = target.getAttr("structureNodeId") as string | undefined;
    if (structureId) return { kind: "structure", id: structureId };
    target = target.getParent();
  }
  return undefined;
}
