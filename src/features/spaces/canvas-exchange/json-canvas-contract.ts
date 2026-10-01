import type { JsonCanvasNodeV1 } from "../contracts/generated/JsonCanvasNodeV1";

/** Local projections of the JSON Canvas wire union, never independent field definitions. */
export type JsonCanvasColor = string;
export type JsonCanvasExtensionFields = Record<string, unknown>;
export type JsonCanvasTextNodeV1 = Extract<JsonCanvasNodeV1, { type: "text" }>;
export type JsonCanvasFileNodeV1 = Extract<JsonCanvasNodeV1, { type: "file" }>;
export type JsonCanvasLinkNodeV1 = Extract<JsonCanvasNodeV1, { type: "link" }>;
export type JsonCanvasGroupNodeV1 = Extract<JsonCanvasNodeV1, { type: "group" }>;
