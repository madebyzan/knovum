import { isDirectMediaResource } from "./direct-media-resource";
import { isMediaPlatformContentUrl } from "./media-platform-url";

export type UrlSourceKind = "media" | "website";

export function classifyUrlSource(url: URL): UrlSourceKind {
  return isDirectMediaResource(url) || isMediaPlatformContentUrl(url) ? "media" : "website";
}
