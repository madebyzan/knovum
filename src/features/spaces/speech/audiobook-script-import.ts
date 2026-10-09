import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { AudiobookSource } from "../contracts/generated/AudiobookSource";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { audiobookDocumentScript } from "./audiobook-document-script";
import { audiobookScript, type ImportedAudiobookScript } from "./audiobook-script";
import { nodeSpeechText } from "./node-speech";

export async function readNodeAudiobookScript(
  node: SpaceCanvasNode,
  role: string,
  jobId: string,
): Promise<ImportedAudiobookScript> {
  const source: AudiobookSource =
    node.data.kind === "file"
      ? await invokeCommand<AudiobookSource>("plugin:spaces|read_audiobook_source", {
          jobId,
          sourceNodeId: node.id,
        })
      : { kind: "text", text: nodeSpeechText(node) };
  if (node.data.kind === "file" && node.data.assetPath?.toLowerCase().endsWith(".json")) {
    if (source.kind !== "text") throw new Error("Structured audiobook source must be text");
    return audiobookScript.parse(JSON.parse(source.text));
  }
  return audiobookScript.parse(audiobookDocumentScript(source, node.data.title, role));
}
