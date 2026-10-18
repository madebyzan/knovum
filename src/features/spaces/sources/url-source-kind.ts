import { directMediaResourceKind } from "./direct-media-resource";
import { mediaPlatformContentKind } from "./media-platform-url";

/**
 * What a link is worth saving as. A media link also says which resource it will become, so the
 * canvas can reserve the right node for it before anything is downloaded.
 */
export type UrlSourceKind = "audio" | "video" | "website";

export function classifyUrlSource(url: URL): UrlSourceKind {
  return directMediaResourceKind(url) ?? mediaPlatformContentKind(url) ?? "website";
}
