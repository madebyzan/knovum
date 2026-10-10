import type { CreateSpeechProjectRequest } from "../contracts/generated/CreateSpeechProjectRequest";
import type { ProjectOutputConfig } from "../contracts/generated/ProjectOutputConfig";
import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "./node-speech";

export function speechProjectInput(
  node: SpaceCanvasNode,
  projectType: NodeSpeechProjectType,
  video?: SpeechMediaSource,
):
  | (Omit<CreateSpeechProjectRequest, "spaceId" | "sourceNodeId"> & { outputConfig: ProjectOutputConfig })
  | undefined {
  const title = [...node.data.title.trim()].slice(0, 200).join("");
  if (projectType === "short_dubbing")
    return { title, projectType, outputConfig: { format: "wav", wavPrecision: "pcm16" } };
  if (projectType === "video_dubbing") {
    if (!video) return;
    return {
      title,
      projectType,
      outputConfig: {
        format: "mp4",
        video,
        background: null,
        backgroundSeparation: null,
        voiceGainDb: 0,
        backgroundGainDb: -12,
        translation: null,
        preserveOriginalOutsideDialogue: false,
        sourceDialogueIntervals: [],
        retainOriginalTrack: true,
      },
    };
  }
  return {
    title,
    projectType,
    outputConfig: {
      format: "m4b",
      sampleRate: 48_000,
      loudnessLufs: -20,
      truePeakDb: -2,
      paragraphPauseMs: 250,
      chapterPauseMs: 1000,
      wavPrecision: "pcm16",
      metadata: { title, author: "", narrator: "", cover: null },
      chapters: [],
      roles: [],
    },
  };
}
