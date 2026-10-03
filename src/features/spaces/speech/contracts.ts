export type VoiceSourceKind = "preset" | "reference" | "designed";
export type SpeechProjectType = "short_dubbing" | "audiobook" | "video_dubbing";

export interface VoiceProfileSummary {
  id: string;
  name: string;
  sourceKind: VoiceSourceKind;
  activeRevisionId: string;
  engineId: string;
  modelSpecId: string;
  language: string;
  presetVoiceId?: string;
  updatedAt: number;
}

export interface SpeechProjectSummary {
  id: string;
  spaceId: string;
  projectType: SpeechProjectType;
  title: string;
  activeRevisionId: string;
  activeRevision: number;
  segmentCount: number;
  generationEpoch: number;
  activeGenerationJobId?: string;
  updatedAt: number;
}

export interface CreatePresetVoiceInput {
  spaceId: string;
  name: string;
  presetVoiceId: string;
  engineId: string;
  modelSpecId: string;
  language: string;
  synthesisConfig: Record<string, unknown>;
}

export interface CreateSpeechProjectInput {
  spaceId: string;
  projectType: SpeechProjectType;
  title: string;
  outputConfig: Record<string, unknown>;
}

export interface SpeechSegment {
  id: string;
  ordinal: number;
  text: string;
  normalizedText: string;
  voiceRevisionId: string;
  language: string;
  state: "pending" | "ready" | "rendering" | "rendered" | "needs_edit" | "failed";
}

export interface SpeechProjectDetail {
  project: SpeechProjectSummary;
  revision: {
    id: string;
    projectId: string;
    revision: number;
    sourceRevision: string;
    outputAssetPath?: string;
    outputSha256?: string;
    outputSampleRate?: number;
    outputSampleCount?: number;
  };
  segments: SpeechSegment[];
}

export interface SaveSpeechProjectRevisionInput {
  projectId: string;
  expectedRevision: number;
  sourceRevision: string;
  pronunciationLexicon: Record<string, string>;
  outputConfig: Record<string, unknown>;
  segments: Array<{
    id: string;
    chapterKey?: string;
    speakerKey?: string;
    text: string;
    voiceRevisionId: string;
    language: string;
    targetStartMs?: number;
    targetEndMs?: number;
  }>;
}

export interface SpeechGenerationProgress {
  stage: "preparing" | "synthesizing" | "rendering" | "done";
  completed: number;
  total: number;
  resources?: { downloaded: number; total?: number; percent?: number };
}

export interface SpeechAudioOutput {
  path: string;
  sha256: string;
  sampleRate: number;
  sampleCount: number;
}

export interface SpeechSegmentRenderOutput extends SpeechAudioOutput {
  segmentId: string;
  cached: boolean;
}

export interface SpeechProjectRenderOutput {
  projectId: string;
  revision: number;
  renders: SpeechSegmentRenderOutput[];
  output: SpeechAudioOutput;
}
