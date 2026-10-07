import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export type NodeSpeechProjectType = "short_dubbing" | "audiobook" | "video_dubbing";

export function nodeSpeechProjectTypes(node: SpaceCanvasNode): readonly NodeSpeechProjectType[] {
  switch (node.data.kind) {
    case "note":
    case "text":
    case "markdown":
      return ["short_dubbing", "audiobook"];
    case "video":
      return ["video_dubbing"];
    case "file":
      return /\.(txt|md|json|doc|docx|pdf|epub)$/iu.test(node.data.assetPath ?? "") ? ["audiobook"] : [];
    default:
      return [];
  }
}

export function nodeSpeechText(node: SpaceCanvasNode): string {
  switch (node.data.kind) {
    case "note":
    case "text":
      return node.data.plainText;
    case "markdown":
      return node.data.text;
    default:
      return "";
  }
}
