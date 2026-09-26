import { Readability } from "@mozilla/readability";
import { take } from "es-toolkit";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { truncateUtf16 } from "../model/text";
import { htmlHeadMetadata } from "./html-head-metadata";
import type { SpaceWebsiteMetadata, SpaceWebsiteSection } from "./metadata";
import { normalizeSpaceUrl, websiteDefaultTitle } from "./url";

type WebDocumentKind = "html" | "json" | "xml" | "text";
type WebpageResponse =
  | { resourceKind: "document"; finalUrl: string; documentKind: WebDocumentKind; content: string }
  | { resourceKind: "image"; finalUrl: string; assetPath: string };

export type WebsiteResource =
  | { kind: "document"; metadata: SpaceWebsiteMetadata }
  | { kind: "image"; finalUrl: string; assetPath: string };

export async function fetchWebsiteResource(url: string): Promise<WebsiteResource> {
  const response = await invoke<WebpageResponse>("plugin:spaces|fetch_webpage", { url });
  const finalUrl = normalizeSpaceUrl(response.finalUrl);
  if (!finalUrl) throw new Error("The webpage response returned an invalid final URL");
  if (response.resourceKind === "image") return { kind: "image", finalUrl, assetPath: response.assetPath };
  const metadata =
    response.documentKind === "html"
      ? htmlMetadata(response.content, finalUrl)
      : textMetadata(response.content, finalUrl, response.documentKind);
  return { kind: "document", metadata };
}

export function cacheWebsiteAsset(url: string): Promise<string> {
  return invoke("plugin:spaces|cache_web_asset", { url });
}

function htmlMetadata(content: string, finalUrl: string): SpaceWebsiteMetadata {
  const document = new DOMParser().parseFromString(content, "text/html");
  const readable = extractReadableContent(document, finalUrl);
  return {
    ...htmlHeadMetadata(document, finalUrl, readable),
    finalUrl,
    ...(readable.preview ? { contentPreview: readable.preview } : {}),
    ...(readable.sections.length ? { knowledgeSections: readable.sections } : {}),
  };
}

function textMetadata(
  content: string,
  finalUrl: string,
  kind: Exclude<WebDocumentKind, "html">,
): SpaceWebsiteMetadata {
  const preview = readableExternalText(content, kind);
  const siteName = websiteDefaultTitle(finalUrl);
  return {
    title: siteName,
    finalUrl,
    siteName,
    ...(preview ? { contentPreview: preview, knowledgeSections: [{ text: preview }] } : {}),
  };
}

function extractReadableContent(
  document: Document,
  finalUrl: string,
): {
  title?: string;
  siteName?: string;
  excerpt?: string;
  preview?: string;
  sections: SpaceWebsiteSection[];
} {
  const readableDocument = document.cloneNode(true) as Document;
  const base = readableDocument.createElement("base");
  base.href = finalUrl;
  readableDocument.head?.prepend(base);
  const article = new Readability(readableDocument).parse();
  const text = article?.textContent?.replace(/\s+/gu, " ").trim();
  const sections = article?.content ? sectionsFromHtml(article.content) : [];
  return {
    ...(article ? readableIdentity(article) : {}),
    ...(text ? { preview: truncateUtf16(text, 12_000) } : {}),
    sections,
  };
}

function readableIdentity(article: NonNullable<ReturnType<Readability["parse"]>>) {
  const title = article.title?.trim();
  const siteName = article.siteName?.trim();
  const excerpt = article.excerpt?.trim();
  return {
    ...(title ? { title } : {}),
    ...(siteName ? { siteName } : {}),
    ...(excerpt ? { excerpt } : {}),
  };
}

function sectionsFromHtml(content: string): SpaceWebsiteSection[] {
  const document = new DOMParser().parseFromString(content, "text/html");
  const sections: SpaceWebsiteSection[] = [];
  let heading: string | undefined;
  let parts: string[] = [];
  const flush = () => {
    const text = parts.join(" ").replace(/\s+/gu, " ").trim();
    if (text) sections.push({ ...(heading ? { heading } : {}), text: truncateUtf16(text, 12_000) });
    parts = [];
  };
  for (const element of document.querySelectorAll("h1,h2,h3,h4,h5,h6,p,li,pre,blockquote")) {
    const text = element.textContent?.replace(/\s+/gu, " ").trim();
    if (!text) continue;
    if (/^H[1-6]$/u.test(element.tagName)) {
      flush();
      heading = truncateUtf16(text, 512);
    } else parts.push(text);
    if (sections.length >= 63) break;
  }
  flush();
  return take(sections, 64);
}

function readableExternalText(content: string, kind: Exclude<WebDocumentKind, "html">) {
  const trimmed = content.trim();
  if (!trimmed) return undefined;
  if (kind === "json") {
    try {
      return truncateUtf16(JSON.stringify(JSON.parse(trimmed), null, 2), 1800);
    } catch {
      return truncateUtf16(trimmed, 1800);
    }
  }
  if (kind === "xml") {
    const document = new DOMParser().parseFromString(trimmed, "application/xml");
    const text = document.documentElement?.textContent?.replace(/\s+/gu, " ").trim();
    if (text) return truncateUtf16(text, 1800);
  }
  return truncateUtf16(trimmed.replace(/\s+/gu, " "), 1800);
}
