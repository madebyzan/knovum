import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";

export const IMAGE_EDIT_TOOL_ORDER = [
  "adjust",
  "crop",
  "transform",
] as const satisfies readonly ImageEditTool[];
