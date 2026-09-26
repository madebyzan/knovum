import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { JsonCanvasDocumentV1 } from "./json-canvas-contract";

export const jsonCanvasCodec = {
  parse: (source: string) => invoke<JsonCanvasDocumentV1>("plugin:spaces|parse_json_canvas", { source }),
  serialize: (document: JsonCanvasDocumentV1) =>
    invoke<string>("plugin:spaces|serialize_json_canvas", { document }),
};
