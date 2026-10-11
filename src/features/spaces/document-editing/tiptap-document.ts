import type { JsonValue } from "../../../shared/contracts/generated/serde_json/JsonValue";

interface TiptapJsonMark {
  type: string;
  attrs?: Record<string, JsonValue>;
}

export interface TiptapJsonNode {
  type: string;
  attrs?: Record<string, JsonValue>;
  content?: TiptapJsonNode[];
  marks?: TiptapJsonMark[];
  text?: string;
}
