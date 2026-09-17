import type { SpaceJsonValue } from "../model/json-value";

interface TiptapJsonMark {
  type: string;
  attrs?: Record<string, SpaceJsonValue>;
}

export interface TiptapJsonNode {
  type: string;
  attrs?: Record<string, SpaceJsonValue>;
  content?: TiptapJsonNode[];
  marks?: TiptapJsonMark[];
  text?: string;
}
