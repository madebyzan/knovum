import { truncateUtf16 } from "../model/text";
import type { SpaceWebsiteMetadata } from "./metadata";
import { websiteDefaultTitle } from "./url";

interface ReadablePageMetadata {
  title?: string;
  siteName?: string;
  excerpt?: string;
}

export function htmlHeadMetadata(
  document: Document,
  finalUrl: string,
  readable: ReadablePageMetadata,
): Omit<SpaceWebsiteMetadata, "finalUrl" | "contentPreview" | "knowledgeSections"> {
  return { ...headText(document, finalUrl, readable), ...headAssets(document, finalUrl) };
}

function headText(document: Document, finalUrl: string, readable: ReadablePageMetadata) {
  const title = pageTitle(document, finalUrl, readable.title);
  const siteName =
    meta(document, "property", "og:site_name") ?? readable.siteName ?? websiteDefaultTitle(finalUrl);
  const description =
    meta(document, "property", "og:description") ??
    meta(document, "name", "twitter:description") ??
    meta(document, "name", "description") ??
    readable.excerpt;
  return {
    title: truncateUtf16(title.trim(), 240),
    siteName: truncateUtf16(siteName.trim(), 120),
    ...(description ? { description: truncateUtf16(description.trim(), 600) } : {}),
  };
}

function pageTitle(document: Document, finalUrl: string, readableTitle?: string) {
  return (
    meta(document, "property", "og:title") ??
    meta(document, "name", "twitter:title") ??
    readableTitle ??
    (document.title.trim() || websiteDefaultTitle(finalUrl))
  );
}

function headAssets(document: Document, finalUrl: string) {
  const canonicalUrl = resolveHttpUrl(
    document.querySelector<HTMLLinkElement>('link[rel~="canonical"]')?.getAttribute("href") ?? undefined,
    finalUrl,
  );
  const previewImageUrl = resolveHttpUrl(
    meta(document, "property", "og:image") ?? meta(document, "name", "twitter:image"),
    finalUrl,
  );
  const faviconUrl = findFaviconUrl(document, finalUrl) ?? new URL("/favicon.ico", finalUrl).href;
  return {
    ...(canonicalUrl ? { canonicalUrl } : {}),
    faviconUrl,
    ...(previewImageUrl ? { previewImageUrl } : {}),
  };
}

function meta(document: Document, attribute: "name" | "property", value: string) {
  return (
    document.querySelector<HTMLMetaElement>(`meta[${attribute}="${value}"]`)?.content.trim() || undefined
  );
}

function findFaviconUrl(document: Document, base: string) {
  for (const link of document.querySelectorAll<HTMLLinkElement>("link[rel]")) {
    if (!link.rel.toLowerCase().split(/\s+/u).includes("icon")) continue;
    const url = resolveHttpUrl(link.getAttribute("href") ?? undefined, base);
    if (url) return url;
  }
  return undefined;
}

function resolveHttpUrl(value: string | undefined, base: string) {
  if (!value) return undefined;
  try {
    const url = new URL(value, base);
    return url.protocol === "http:" || url.protocol === "https:" ? url.href : undefined;
  } catch {
    return undefined;
  }
}
