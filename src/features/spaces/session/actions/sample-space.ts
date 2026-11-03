import { i18n } from "../../../../foundation/i18n/instance";
import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type {
  CanvasImportReceipt,
  CanvasJsonImportRequest,
} from "../../canvas-exchange/json-canvas-import-contract";
import { createAndTrackSpace } from "../../workspace/space-creation";
import type { SpaceSessionContext } from "../../workspace/space-session-context";
import { openSpace } from "../../workspace/workbench-navigation-actions";
import type { SpaceState } from "../session-state";

/**
 * Gives a new library a worked example to explore instead of an empty canvas. It is an ordinary
 * Space built by the ordinary import, so it behaves like any other and is deleted like any other.
 */
export async function seedSampleSpace(
  context: SpaceSessionContext,
  state: SpaceState,
  importJsonCanvas: (request: CanvasJsonImportRequest) => Promise<CanvasImportReceipt>,
) {
  const space = await createAndTrackSpace(context, i18n.global.t("spaces.space.sampleTitle"), "space");
  await openSpace(context, space.id);
  const sourcePath = await invokeCommand<string>("plugin:spaces|sample_space_source", {
    locale: i18n.global.locale.value,
  });
  await importJsonCanvas({ sourcePath, baseRevision: state.editSnapshot.value.editRevision });
}
