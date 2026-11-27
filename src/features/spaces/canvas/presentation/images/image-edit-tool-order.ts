import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";

/** High-frequency editing tool kept directly on the primary image toolbar. */
export const PRIMARY_IMAGE_EDIT_TOOLS = ["crop"] as const satisfies readonly ImageEditTool[];

/** Secondary image editing tools housed inside the image overflow menu. */
export const SECONDARY_IMAGE_EDIT_TOOLS = ["adjust", "transform"] as const satisfies readonly ImageEditTool[];
