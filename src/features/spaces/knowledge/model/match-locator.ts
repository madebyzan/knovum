import type { KnowledgeSearchMatch } from "./knowledge-search-match";
import type { KnowledgeChunkKind, KnowledgeLocation } from "./schema";

export type SpaceMatchLocator =
  | { kind: "node" }
  | { kind: "text"; location: KnowledgeLocation }
  | { kind: "media-time-range"; startMs: number; endMs: number }
  | { kind: "image-description" }
  | { kind: "video-first-frame" };

export function matchLocatorFor(match: KnowledgeSearchMatch): SpaceMatchLocator {
  return resolveLocator(match.chunkKind, match.locator.location);
}

function resolveLocator(kind: KnowledgeChunkKind, location: KnowledgeLocation): SpaceMatchLocator {
  if (kind === "image_description") return { kind: "image-description" };
  if (kind === "video_first_frame") return { kind: "video-first-frame" };
  if (location.kind === "media")
    return { kind: "media-time-range", startMs: location.startMs, endMs: location.endMs };
  if (kind === "node" || location.kind === "node") return { kind: "node" };
  return { kind: "text", location };
}
