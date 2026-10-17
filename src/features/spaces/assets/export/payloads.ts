import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { formatExportMarkdown } from "../markdown-document";

/**
 * A format the Renderer serializes from content it already holds. Everything
 * that needs the managed bytes - pictures, recordings, document rendering - is
 * produced by Rust instead, which is also where those formats are declared.
 */
export interface PayloadExport {
  extension: string;
  text: () => string;
}

export function payloadExports(data: SpaceNodePayload): readonly PayloadExport[] {
  // A note is editable document content: the document engine renders it.
  if (isSpaceManagedAssetPayload(data) || data.kind === "note") return [];
  if (data.kind === "text") return [{ extension: "txt", text: () => data.plainText }];
  if (data.kind === "markdown")
    return [{ extension: "md", text: () => formatExportMarkdown(data.title, data.text) }];
  if (data.kind === "website") return [{ extension: "md", text: () => websiteMarkdown(data) }];
  if (data.kind === "credential") return [json(() => credentialMetadata(data))];
  if (data.kind === "object") return [json(() => objectDocument(data))];
  if (data.kind === "portal") return [json(() => portalDocument(data))];
  data satisfies never;
  throw new Error("Unsupported node export");
}

export function json(value: () => unknown): PayloadExport {
  return { extension: "json", text: () => `${JSON.stringify(value(), undefined, 2)}\n` };
}

function websiteMarkdown(data: Extract<SpaceNodePayload, { kind: "website" }>) {
  const lines = [`# ${data.title}`, "", `Source: ${data.url}`];
  if (data.canonicalUrl && data.canonicalUrl !== data.url) lines.push(`Canonical: ${data.canonicalUrl}`);
  if (data.siteName) lines.push(`Site: ${data.siteName}`);
  if (data.description) lines.push("", data.description);
  if (data.contentPreview) lines.push("", data.contentPreview);
  for (const section of data.websiteSections ?? []) {
    if (section.heading) lines.push("", `## ${section.heading}`);
    lines.push("", section.text);
  }
  return `${lines.join("\n")}\n`;
}

export function credentialMetadata(data: Extract<SpaceNodePayload, { kind: "credential" }>) {
  return {
    kind: data.kind,
    title: data.title,
    credentialKind: data.credentialKind,
    configured: data.credentialConfigured === true,
  };
}

export function objectDocument(data: Extract<SpaceNodePayload, { kind: "object" }>) {
  return {
    kind: data.kind,
    title: data.title,
    objectTypeId: data.objectTypeId,
    status: data.status,
    summary: data.summary,
    properties: data.properties,
    tags: data.tags,
  };
}

export function portalDocument(data: Extract<SpaceNodePayload, { kind: "portal" }>) {
  return {
    kind: data.kind,
    title: data.title,
    targetSpaceId: data.targetSpaceId,
    targetSceneId: data.targetSceneId,
    targetNodeId: data.targetNodeId,
    targetStructureNodeId: data.targetStructureNodeId,
  };
}
