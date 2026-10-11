import type { SpaceNodePayload } from "../../../model/nodes/node-payload";

type CanvasNodeWorkKind = "asset" | "derivation" | "website" | "analysis" | "edit";
type ResourceNode = Extract<SpaceNodePayload, { kind: "image" | "video" | "audio" | "file" | "code" }>;

export interface CanvasNodeWork {
  kind: CanvasNodeWorkKind;
  phase: "active" | "failed";
  surface: "content" | "chrome" | "selection";
  labelKey: string;
}

export function nodePayloadWork(data: SpaceNodePayload): CanvasNodeWork | undefined {
  if (data.kind === "markdown") return markdownWork(data.derivationState, Boolean(data.text));
  if ("processingState" in data) return assetWork(data);
  if (data.kind === "website") return websiteWork(data.metadataState);
  return undefined;
}

function markdownWork(
  state: "pending" | "failed" | undefined,
  hasContent: boolean,
): CanvasNodeWork | undefined {
  if (!state) return undefined;
  return {
    kind: "derivation",
    phase: state === "pending" ? "active" : "failed",
    surface: hasContent ? "chrome" : "content",
    labelKey: state === "pending" ? "spaces.creation.document.generating" : "spaces.creation.document.failed",
  };
}

function assetWork(data: ResourceNode): CanvasNodeWork | undefined {
  const state = data.processingState;
  if (!state) return undefined;
  if (state === "failed") {
    return {
      kind: "asset",
      phase: "failed",
      surface: assetContentAvailable(data) ? "chrome" : "content",
      labelKey: "spaces.processing.failed",
    };
  }
  return {
    kind: "asset",
    phase: "active",
    surface: state === "importing" || !assetContentAvailable(data) ? "content" : "chrome",
    labelKey: state === "importing" ? "spaces.importExport.progress.resource" : processingLabel(data.kind),
  };
}

function processingLabel(kind: ResourceNode["kind"]): string {
  if (kind === "image") return "spaces.processing.imageProgress";
  if (kind === "video" || kind === "audio") return "spaces.processing.mediaProgress";
  return "spaces.importExport.progress.processing";
}

function assetContentAvailable(data: ResourceNode): boolean {
  switch (data.kind) {
    case "image":
    case "video":
    case "audio":
      return Boolean(data.sourceRevision);
    case "file":
      return Boolean(data.previewPath || data.previewText || data.extractedText || data.entries?.length);
    case "code":
      return Boolean(data.previewText);
  }
}

function websiteWork(state: "idle" | "loading" | "ready" | "failed" | undefined): CanvasNodeWork | undefined {
  if (state === "loading") {
    return {
      kind: "website",
      phase: "active",
      surface: "chrome",
      labelKey: "spaces.importExport.website.loading",
    };
  }
  if (state === "failed") {
    return {
      kind: "website",
      phase: "failed",
      surface: "chrome",
      labelKey: "spaces.importExport.website.previewUnavailable",
    };
  }
  return undefined;
}

export function canvasNodeWork(
  data: SpaceNodePayload,
  activity: { mediaAnalysis: boolean; imageEditApplying: boolean },
): CanvasNodeWork | undefined {
  if (activity.imageEditApplying && data.kind === "image") {
    return {
      kind: "edit",
      phase: "active",
      surface: "selection",
      labelKey: "spaces.processing.applyingEdit",
    };
  }
  const payloadWork = nodePayloadWork(data);
  if (payloadWork) return payloadWork;
  if (activity.mediaAnalysis) {
    return {
      kind: "analysis",
      phase: "active",
      surface: "chrome",
      labelKey: "spaces.media.analysis.inProgress",
    };
  }
  return undefined;
}
