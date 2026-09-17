/** JSON Canvas 1.0: https://jsoncanvas.org/spec/1.0/ */
export type JsonCanvasColor = string;
export type JsonCanvasNodeSide = "top" | "right" | "bottom" | "left";
export type JsonCanvasEdgeEnd = "none" | "arrow";
export type JsonCanvasExtensionFields = Record<string, unknown>;

export interface JsonCanvasNodeFrameV1 extends JsonCanvasExtensionFields {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color?: JsonCanvasColor;
}

export type JsonCanvasTextNodeV1 = JsonCanvasNodeFrameV1 & { type: "text"; text: string };
export type JsonCanvasFileNodeV1 = JsonCanvasNodeFrameV1 & {
  type: "file";
  file: string;
  subpath?: string;
};
export type JsonCanvasLinkNodeV1 = JsonCanvasNodeFrameV1 & { type: "link"; url: string };
export type JsonCanvasGroupNodeV1 = JsonCanvasNodeFrameV1 & {
  type: "group";
  label?: string;
  background?: string;
  backgroundStyle?: "cover" | "ratio" | "repeat";
};

export type JsonCanvasNodeV1 =
  | JsonCanvasTextNodeV1
  | JsonCanvasFileNodeV1
  | JsonCanvasLinkNodeV1
  | JsonCanvasGroupNodeV1;

export interface JsonCanvasEdgeV1 extends JsonCanvasExtensionFields {
  id: string;
  fromNode: string;
  fromSide?: JsonCanvasNodeSide;
  fromEnd?: JsonCanvasEdgeEnd;
  toNode: string;
  toSide?: JsonCanvasNodeSide;
  toEnd?: JsonCanvasEdgeEnd;
  color?: JsonCanvasColor;
  label?: string;
}

export interface JsonCanvasDocumentV1 extends JsonCanvasExtensionFields {
  nodes?: JsonCanvasNodeV1[];
  edges?: JsonCanvasEdgeV1[];
}
