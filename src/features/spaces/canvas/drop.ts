import type { FileDrop } from "../../../foundation/file-drop/events";
import { i18n } from "../../../foundation/i18n/instance";
import { importSpaceBatch } from "../assets/import-batch";
import { spaceImportContext, type SpaceImportStore } from "../assets/import-context";
import type { useSpaceStore } from "../session/space-store";
import type { CanvasPoint } from "./contracts/canvas-editor-port";

type SpaceStore = SpaceImportStore &
  Pick<ReturnType<typeof useSpaceStore>, "focusedEditorId" | "requestReveal">;

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
