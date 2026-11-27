const VIDEO_EXTENSION_PATTERN = /\.(?:mp4|webm|mov|m4v)$/iu;
const PENDING_MEDIA_DIRECTIVE_PATTERN = /^knovum-gen:\/\/(image|video)(?:\?|$)/iu;

export type PendingMediaKind = "image" | "video";

export function isVideoMediaUrl(url: string): boolean {
  const trimmed = url.trim();
  if (!trimmed) return false;
  const withoutQuery = trimmed.split(/[?#]/u)[0] ?? trimmed;
  try {
    return VIDEO_EXTENSION_PATTERN.test(decodeURIComponent(withoutQuery));
  } catch {
    return VIDEO_EXTENSION_PATTERN.test(withoutQuery);
  }
}

export function parsePendingMediaDirective(url: string): { readonly kind: PendingMediaKind } | undefined {
  const match = PENDING_MEDIA_DIRECTIVE_PATTERN.exec(url.trim());
  if (!match?.[1]) return undefined;
  return { kind: match[1].toLowerCase() as PendingMediaKind };
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

/**
 * Renders specialized Markdown media nodes when the destination is either a pending AI
 * generation directive (`knovum-gen://image|video?...`) or a native web video URL.
 * Returns `undefined` for ordinary images so markdown-it renders a standard `<img>`.
 */
export function renderMarkdownMediaHtml(src: string, alt: string): string | undefined {
  const caption = alt.trim();
  const pending = parsePendingMediaDirective(src);
  if (pending) {
    const figcaption = caption ? `<figcaption>${escapeHtml(caption)}</figcaption>` : "";
    return `<figure class="ui-md-media-pending" data-media-kind="${pending.kind}"><div class="ui-md-media-shimmer" aria-hidden="true"></div>${figcaption}</figure>`;
  }
  if (isVideoMediaUrl(src)) {
    const figcaption = caption ? `<figcaption>${escapeHtml(caption)}</figcaption>` : "";
    return `<figure class="ui-md-video"><video controls playsinline preload="metadata" src="${escapeHtml(src)}"></video>${figcaption}</figure>`;
  }
  return undefined;
}
