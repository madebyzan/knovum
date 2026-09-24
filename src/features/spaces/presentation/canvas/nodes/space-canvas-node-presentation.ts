import {
  AudioLines,
  Boxes,
  Captions,
  Code2,
  File,
  Globe2,
  Image,
  KeyRound,
  NotebookPen,
  Type,
  Sparkles,
  Video,
} from "@lucide/vue";
import type { Component } from "vue";

import { spaceNodePreviewSource } from "../../../assets/node-preview-source";
import { mediaProviderDisplayName } from "../../../media/provider-display-name";
import type { SpaceNodeKind, SpaceNodePayload } from "../../../model/nodes/node-payload";
import { spaceNodeSummary } from "./space-canvas-node-content";

export interface CanvasNodeOverlayPresentation {
  icon: Component;
  title: string;
  subtitle?: string;
  body: string;
  previewSource: string;
  mediaSource: string;
  mediaKind?: "audio" | "video";
  mono: boolean;
}

export function spaceCanvasNodePresentation(data: SpaceNodePayload): CanvasNodeOverlayPresentation {
  const subtitle = nodeSubtitle(data);
  const mediaReady = (data.kind === "video" || data.kind === "audio") && Boolean(data.sourceRevision);
  return {
    icon: NODE_ICONS[data.kind],
    title: data.title,
    ...(subtitle ? { subtitle } : {}),
    body: spaceNodeSummary(data) ?? "",
    previewSource: spaceNodePreviewSource(data),
    mediaSource: mediaReady && (data.kind === "video" || data.kind === "audio") ? data.assetPath : "",
    ...(data.kind === "video" || data.kind === "audio" ? { mediaKind: data.kind } : {}),
    mono: data.kind === "code",
  };
}

const NODE_ICONS: Readonly<Record<SpaceNodeKind, Component>> = {
  note: NotebookPen,
  text: Type,
  markdown: File,
  website: Globe2,
  image: Image,
  video: Video,
  audio: AudioLines,
  file: File,
  code: Code2,
  subtitle: Captions,
  credential: KeyRound,
  object: Sparkles,
  portal: Boxes,
};

const NODE_LABELS: Readonly<Record<SpaceNodeKind, string>> = {
  note: "Note",
  text: "Text",
  markdown: "Markdown",
  website: "Website",
  image: "Image",
  video: "Video",
  audio: "Audio",
  file: "File",
  code: "Code",
  subtitle: "Subtitles",
  credential: "Credential",
  object: "Knowledge object",
  portal: "Portal",
};

function nodeSubtitle(data: SpaceNodePayload) {
  const subtitle = mediaSubtitle(data) ?? fileSubtitle(data) ?? knowledgeSubtitle(data);
  return subtitle === data.title ? undefined : (subtitle ?? NODE_LABELS[data.kind]);
}

function mediaSubtitle(data: SpaceNodePayload) {
  switch (data.kind) {
    case "website":
      return data.siteName ?? data.url;
    case "image":
      return data.fileName;
    case "video":
      return mediaIdentity(data);
    case "audio":
      return mediaIdentity(data);
    default:
      return undefined;
  }
}

function mediaIdentity(data: Extract<SpaceNodePayload, { kind: "audio" | "video" }>) {
  const identity = [data.origin?.provider && mediaProviderDisplayName(data.origin.provider), data.creator]
    .filter(Boolean)
    .join(" · ");
  return identity || data.fileName;
}

function fileSubtitle(data: SpaceNodePayload) {
  switch (data.kind) {
    case "file":
      return data.entries ? `${data.entries.length} entries` : data.fileName;
    case "code":
      return data.previewLanguage;
    case "subtitle":
      return `${data.cues.length} cues`;
    default:
      return undefined;
  }
}

function knowledgeSubtitle(data: SpaceNodePayload) {
  switch (data.kind) {
    case "credential":
      return data.credentialKind;
    case "object":
      return data.objectTypeId;
    case "portal":
      return data.targetSceneId ? "Linked scene" : "Linked space";
    default:
      return undefined;
  }
}
