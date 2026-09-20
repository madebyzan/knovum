import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { StructureCommand } from "../../../organization/structure/commands/contract";
import { indexStructureSnapshot } from "../../../organization/structure/structure-context";
import { projectStructure } from "../../../organization/structure/structure-projection";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasPoint } from "../../contract/canvas-editor-port";
import { SECTION_DEFAULT_SIZE, SECTION_MINIMUM_SIZE } from "../../contract/section-metrics";
export function useSectionDrawing(options: {
  document(): SpaceEditorDocument;
  zoom(): number;
  exitScope(): void;
  execute(command: StructureCommand): Promise<void>;
  complete(): void;
  reportError(error: unknown): void;
}) {
  const { t } = useI18n();
  const draft = shallowRef<{ start: CanvasPoint; end: CanvasPoint }>();
  const pending = shallowRef(false);
  const click = computed(
    () =>
      !draft.value ||
      Math.hypot(draft.value.end.x - draft.value.start.x, draft.value.end.y - draft.value.start.y) *
        options.zoom() <
        5,
  );
  const bounds = computed(() => {
    const value = draft.value;
    if (!value) return undefined;
    if (click.value) return { ...value.start, ...SECTION_DEFAULT_SIZE };
    const dx = value.end.x - value.start.x,
      dy = value.end.y - value.start.y;
    const width = Math.max(SECTION_MINIMUM_SIZE.width, Math.abs(dx)),
      height = Math.max(SECTION_MINIMUM_SIZE.height, Math.abs(dy));
    return {
      x: dx < 0 ? value.start.x - width : value.start.x,
      y: dy < 0 ? value.start.y - height : value.start.y,
      width,
      height,
    };
  });
  const candidates = computed(() => {
    const region = bounds.value;
    if (!region || click.value) return [];
    const context = indexStructureSnapshot(options.document());
    const projection = projectStructure(context);
    return (context.tree.children.get(null) ?? [])
      .filter((item) => {
        if (item.kind === "section" || item.locked) return false;
        const rect = projection.bounds.get(item.id)!;
        return (
          rect.x >= region.x &&
          rect.y >= region.y &&
          rect.x + rect.width <= region.x + region.width &&
          rect.y + rect.height <= region.y + region.height
        );
      })
      .map((item) => item.id);
  });
  function begin(point: CanvasPoint) {
    if (!pending.value) {
      options.exitScope();
      draft.value = { start: point, end: point };
    }
  }
  function move(point: CanvasPoint) {
    if (draft.value) draft.value = { ...draft.value, end: point };
  }
  function cancel() {
    draft.value = undefined;
  }
  async function finish() {
    const region = bounds.value;
    if (!region || pending.value) return;
    const ids = candidates.value;
    cancel();
    pending.value = true;
    try {
      await options.execute({
        kind: "draw-section",
        id: crypto.randomUUID(),
        title: t("spaces.structure.untitledSection"),
        bounds: region,
        ids,
      });
      options.complete();
    } catch (error) {
      options.reportError(error);
    } finally {
      pending.value = false;
    }
  }
  return { bounds, candidates, pending, begin, move, cancel, finish };
}
