import type { SpaceNodeKind } from "../../../model/nodes/node-payload";

export type CanvasNodeProjectionKind =
  | "code-lines"
  | "fixed"
  | "markdown-blocks"
  | "object-fields"
  | "resource-entries"
  | "subtitle-cues"
  | "tiptap-blocks"
  | "website-fields";

export interface CanvasNodeDetailStrategy {
  projection: CanvasNodeProjectionKind;
}

export const CANVAS_NODE_DETAIL_STRATEGIES = {
  note: { projection: "tiptap-blocks" },
  text: { projection: "tiptap-blocks" },
  markdown: { projection: "markdown-blocks" },
  subtitle: { projection: "subtitle-cues" },
  website: { projection: "website-fields" },
  image: { projection: "fixed" },
  video: { projection: "fixed" },
  audio: { projection: "fixed" },
  file: { projection: "resource-entries" },
  code: { projection: "code-lines" },
  credential: { projection: "fixed" },
  object: { projection: "object-fields" },
  portal: { projection: "fixed" },
} as const satisfies Readonly<Record<SpaceNodeKind, CanvasNodeDetailStrategy>>;
