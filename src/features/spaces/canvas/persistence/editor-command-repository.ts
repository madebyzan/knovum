import { invoke } from "@tauri-apps/api/core";

import type { ExecuteSpaceEditRequest, ReplaySpaceEditRequest } from "../contract/editor-command";
import {
  editorWorkspaceSchema,
  spaceEditResultSchema,
  type SpaceEditResult,
} from "../contract/editor-result";

export class EditorOutcomeUnknownError extends Error {
  constructor(
    readonly mutationId: string,
    cause: unknown,
  ) {
    super(
      `The outcome of edit ${mutationId} could not be confirmed. Do not repeat the operation with a new identity.`,
      { cause },
    );
    this.name = "EditorOutcomeUnknownError";
  }
}

export async function loadEditorWorkspace(spaceId: string) {
  return editorWorkspaceSchema.parse(
    await invoke<unknown>("plugin:spaces|load_editor_workspace", { spaceId }),
  );
}
export function executeSpaceEdit(spaceId: string, request: ExecuteSpaceEditRequest) {
  return submit(spaceId, request.mutationId, () =>
    invoke<unknown>("plugin:spaces|execute_space_edit", {
      request: {
        spaceId,
        mutationId: request.mutationId,
        baseRevision: request.baseRevision,
        command: request.command,
      },
    }),
  );
}
export function replaySpaceEdit(spaceId: string, request: ReplaySpaceEditRequest) {
  return submit(spaceId, request.mutationId, () =>
    invoke<unknown>("plugin:spaces|replay_space_edit", {
      request: {
        spaceId,
        mutationId: request.mutationId,
        baseRevision: request.baseRevision,
        receiptId: request.receiptId,
        direction: request.direction,
      },
    }),
  );
}
async function submit(
  spaceId: string,
  mutationId: string,
  operation: () => Promise<unknown>,
): Promise<SpaceEditResult> {
  let received = false;
  try {
    const result = await operation();
    received = true;
    return spaceEditResultSchema.parse(result);
  } catch (cause) {
    let receipt: unknown;
    try {
      receipt = await invoke<unknown>("plugin:spaces|query_space_edit_receipt", {
        request: { spaceId, mutationId },
      });
    } catch (queryError) {
      throw new EditorOutcomeUnknownError(mutationId, new AggregateError([cause, queryError]));
    }
    if (receipt !== null) {
      try {
        return spaceEditResultSchema.parse(receipt);
      } catch (error) {
        throw new EditorOutcomeUnknownError(mutationId, error);
      }
    }
    // Rust's rejected Result is a string. A transport or decode failure is not proof of rollback.
    if (!received && typeof cause === "string") throw new Error(cause);
    throw new EditorOutcomeUnknownError(mutationId, cause);
  }
}
