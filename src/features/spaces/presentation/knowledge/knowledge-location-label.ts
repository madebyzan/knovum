import type { KnowledgeLocation, KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import { formatDurationMs } from "../../media/format-duration";

export function knowledgeLocationLabel(locator: KnowledgeLocator): string {
  return knowledgeLocationLabelFromLocation(locator.location);
}

export function knowledgeLocationLabelFromLocation(location: KnowledgeLocation) {
  if (location.kind === "media") return formatDurationMs(location.startMs);
  if (location.kind === "pdf") return `PDF · p.${location.page}`;
  if (location.kind === "word") return wordLocationLabel(location);
  if (location.kind === "slides") return `Slides · ${location.slide}`;
  if (location.kind === "sheet") return sheetLocationLabel(location);
  if (location.kind === "markdown") return markdownLocationLabel(location);
  if (location.kind === "website") return websiteLocationLabel(location);
  if (location.kind === "code") return codeLocationLabel(location);
  return location.kind === "note" ? "Note" : "Space content";
}

function wordLocationLabel(location: Extract<KnowledgeLocation, { kind: "word" }>) {
  const label = location.headingPath.length
    ? `Word · ${location.headingPath.join(" / ")}`
    : `Word · §${location.section + 1}`;
  const parts = {
    header: "Header",
    footer: "Footer",
    first_page_header: "First-page header",
    first_page_footer: "First-page footer",
    even_page_header: "Even-page header",
    even_page_footer: "Even-page footer",
  };
  return location.part ? `${label} · ${parts[location.part]}` : label;
}

function sheetLocationLabel(location: Extract<KnowledgeLocation, { kind: "sheet" }>) {
  return `Sheet · ${location.sheet}${location.range ? ` · ${location.range}` : ""}`;
}

function markdownLocationLabel(location: Extract<KnowledgeLocation, { kind: "markdown" }>) {
  return location.heading ? `Markdown · ${location.heading}` : "Markdown";
}

function websiteLocationLabel(location: Extract<KnowledgeLocation, { kind: "website" }>) {
  return location.section ? `Website · ${location.section}` : "Website";
}

function codeLocationLabel(location: Extract<KnowledgeLocation, { kind: "code" }>) {
  const prefix = location.path ? `${location.path} · ` : "";
  return `${prefix}L${location.lineStart}–${location.lineEnd}`;
}
