import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceEditHistoryReceipt } from "../contracts/generated/SpaceEditHistoryReceipt";
import { invokeConfirmedMutation } from "../organization/confirmed-mutation";
import type { ExecuteSpaceEditRequest, SpaceEditReceipt } from "../organization/space-edit-command";

/** Typed IPC boundary. Rust owns revision checks, mutations and committed snapshots. */
export const spaceEditOperations = {
  async load(spaceId: string): Promise<SpaceEditSnapshot> {
    const snapshot = await invoke<unknown>("plugin:spaces|load_space_edit_snapshot", { spaceId });
    return parseSpaceEditSnapshot(snapshot);
  },
  execute(request: ExecuteSpaceEditRequest): Promise<SpaceEditReceipt> {
    return invokeConfirmedMutation<SpaceEditReceipt>(
      "plugin:spaces|execute_space_edit",
      { request },
      request.mutationId,
    );
  },
  undo(spaceId: string, expectedEditRevision: number): Promise<SpaceEditHistoryReceipt> {
    return invoke<SpaceEditHistoryReceipt>("plugin:spaces|undo_space_edit", {
      request: { spaceId, expectedEditRevision },
    });
  },
  redo(spaceId: string, expectedEditRevision: number): Promise<SpaceEditHistoryReceipt> {
    return invoke<SpaceEditHistoryReceipt>("plugin:spaces|redo_space_edit", {
      request: { spaceId, expectedEditRevision },
    });
  },
};
