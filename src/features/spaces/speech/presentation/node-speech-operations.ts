import { BookOpen, Clapperboard, Mic2, Music2, Speech } from "@lucide/vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeSpeechProjectTypes, type NodeSpeechProjectType } from "../node-speech";

export type NodeSpeechOperation = NodeSpeechProjectType | "clone" | "separate";

export function nodeSpeechOperationIds(node: SpaceCanvasNode): readonly NodeSpeechOperation[] {
  const projects = nodeSpeechProjectTypes(node);
  return node.data.kind === "audio" || node.data.kind === "video"
    ? [...projects, "clone", "separate"]
    : projects;
}

export const NODE_SPEECH_OPERATIONS = {
  short_dubbing: { surface: "inspector", icon: Speech, label: "spaces.speech.project.generateFromNode" },
  audiobook: { surface: "inspector", icon: BookOpen, label: "spaces.speech.project.types.audiobook" },
  video_dubbing: {
    surface: "inspector",
    icon: Clapperboard,
    label: "spaces.speech.project.types.videoDubbing",
  },
  clone: { surface: "compact-editor", icon: Mic2, label: "spaces.speech.voice.clone.fromReference" },
  separate: { surface: "compact-editor", icon: Music2, label: "spaces.speech.separation.title" },
} as const;
