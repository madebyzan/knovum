import { BookOpen, Clapperboard, Mic2, Music2, Speech } from "@lucide/vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeSpeechProjectTypes, type NodeSpeechProjectType } from "../../speech/node-speech";

export type NodeSpeechOperation = NodeSpeechProjectType | "clone";
export type NodeSpeechAction = NodeSpeechOperation | "separate";

export function nodeSpeechOperationIds(node: SpaceCanvasNode): readonly NodeSpeechAction[] {
  const projects = nodeSpeechProjectTypes(node);
  return node.data.kind === "audio" || node.data.kind === "video"
    ? [...projects, "clone", "separate"]
    : projects;
}

export const nodeSpeechOperations = {
  short_dubbing: { icon: Speech, label: "spaces.speech.project.generateFromNode" },
  audiobook: { icon: BookOpen, label: "spaces.speech.project.types.audiobook" },
  video_dubbing: { icon: Clapperboard, label: "spaces.speech.project.types.videoDubbing" },
  clone: { icon: Mic2, label: "spaces.speech.voice.clone" },
  separate: { icon: Music2, label: "spaces.speech.separation.title" },
};
