import type { JsonCanvasNodeV1 } from "../contracts/generated/JsonCanvasNodeV1";
export type { JsonCanvasDocumentV1 } from "../contracts/generated/JsonCanvasDocumentV1";
export type { JsonCanvasNodeV1 } from "../contracts/generated/JsonCanvasNodeV1";
export type { JsonCanvasEdgeV1 } from "../contracts/generated/JsonCanvasEdgeV1";
export type { NodeFrameV1 as JsonCanvasNodeFrameV1 } from "../contracts/generated/NodeFrameV1";
export type { NodeSideV1 as JsonCanvasNodeSide } from "../contracts/generated/NodeSideV1";
export type { EdgeEndV1 as JsonCanvasEdgeEnd } from "../contracts/generated/EdgeEndV1";

/** Local projections of the JSON Canvas wire union, never independent field definitions. */
export type JsonCanvasColor = string;
export type JsonCanvasExtensionFields = Record<string, unknown>;
export type JsonCanvasTextNodeV1 = Extract<JsonCanvasNodeV1, { type: "text" }>;
export type JsonCanvasFileNodeV1 = Extract<JsonCanvasNodeV1, { type: "file" }>;
export type JsonCanvasLinkNodeV1 = Extract<JsonCanvasNodeV1, { type: "link" }>;
export type JsonCanvasGroupNodeV1 = Extract<JsonCanvasNodeV1, { type: "group" }>;
