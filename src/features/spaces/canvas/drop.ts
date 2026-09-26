import type { FileDrop } from "../../../foundation/file-drop/file-drop";
import { i18n } from "../../../foundation/i18n/i18n";
import { importSpaceBatch } from "../assets/import-batch";
import { spaceImportContext } from "../assets/import-context";
import type { useSpaceStore } from "../session/space-store";
import type { CanvasPoint } from "./contract/canvas-editor";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export async function importDroppedFiles(payload: FileDrop, store: SpaceStore, origin: CanvasPoint) {
  if (store.focusedEditorId) return;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const labels = {
    title: i18n.global.t("spaces.importExport.progress.resource"),
    preparing: i18n.global.t("spaces.importExport.progress.preparing"),
  };
  const outcomes = await importSpaceBatch({
    context: spaceImportContext(store),
    spaceId,
    requests: payload.paths.map((path) => ({ key: path, path, sourceAccess: "scoped" as const })),
    origin,
    labels,
  });
  const added = outcomes.flatMap((outcome) => (outcome.status === "imported" ? outcome.nodes : []));
  if (store.activeSpaceId === spaceId && added.length) store.requestReveal(added.map((node) => node.id));
  const failures = outcomes.filter((outcome) => outcome.status === "failed");
  if (failures.length) {
    if (failures.length === 1) throw failures[0]!.error;
    throw new AggregateError(
      failures.map(({ error }) => error),
      i18n.global.t("spaces.importExport.importFailed"),
    );
  }
}
