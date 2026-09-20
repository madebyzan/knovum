export type StructureErrorCode =
  | "missing-item"
  | "duplicate-identity"
  | "invalid-parent"
  | "invalid-order"
  | "cycle"
  | "depth-limit"
  | "empty-group"
  | "invalid-layout"
  | "invalid-reference"
  | "invalid-selection"
  | "different-parents"
  | "locked"
  | "history-conflict";

export class StructureError extends Error {
  constructor(
    readonly code: StructureErrorCode,
    message: string,
    readonly itemIds: readonly string[] = [],
  ) {
    super(message);
    this.name = "StructureError";
  }
}
