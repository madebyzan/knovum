import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ExecuteSpaceEditRequest } from "../contracts/generated/ExecuteSpaceEditRequest";
import type { SpaceEditHistoryReceipt } from "../contracts/generated/SpaceEditHistoryReceipt";
import type { SpaceEditReceipt } from "../contracts/generated/SpaceEditReceipt";
import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { invokeConfirmedMutation } from "./confirmed-mutation";

/** Typed IPC boundary. Rust owns revision checks, mutations and committed snapshots. */
export const spaceEditOperations = {
  async load(spaceId: string): Promise<SpaceEditSnapshot> {
    const snapshot = await invokeCommand<unknown>("plugin:spaces|load_space_edit_snapshot", { spaceId });
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
    return invokeCommand<SpaceEditHistoryReceipt>("plugin:spaces|undo_space_edit", {
      request: { spaceId, expectedEditRevision },
    });
  },
  redo(spaceId: string, expectedEditRevision: number): Promise<SpaceEditHistoryReceipt> {
    return invokeCommand<SpaceEditHistoryReceipt>("plugin:spaces|redo_space_edit", {
      request: { spaceId, expectedEditRevision },
    });
  },
};
