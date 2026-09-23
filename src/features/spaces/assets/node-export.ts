import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload, SpaceSubtitlePayload } from "../model/nodes/node-payload";
import type { StructureIndex } from "../model/structure/structure-index";
import { validateStructureSnapshot } from "../model/structure/structure-validation";
import { exportSpaceAsset, exportSpaceTextFile, safeExportFileName, type SpaceTextExport } from "./export";
import { exportEditedImage } from "./edited-image-export";
import { formatExportMarkdown } from "./markdown-document";

export async function exportSpaceNode(spaceId: string, node: SpaceCanvasNode): Promise<boolean> {
  if (node.data.kind === "image") return exportEditedImage(spaceId, node.id, node.data);
  if (isSpaceManagedAssetPayload(node.data)) return exportSpaceAsset(node.data);
  return exportSpaceTextFile(nodeTextExport(node.data));
}

export async function exportSpaceSection(
  snapshot: SpaceEditSnapshot,
  nodes: readonly SpaceCanvasNode[],
  sectionId: string,
): Promise<boolean> {
  const index = validateStructureSnapshot(snapshot.structure, snapshot.layouts);
  const section = index.structureById.get(sectionId);
  if (section?.kind !== "section") throw new Error(`Section not found: ${sectionId}`);
  const nodesById = new Map(nodes.map((node) => [node.id, node]));
  const document = {
    version: 1,
    kind: "section",
    title: section.title,
    members: sectionOutlineMembers(sectionId, index, nodesById),
  };
  return exportSpaceTextFile(jsonExport(safeExportFileName(section.title) || "section", document));
}

function nodeTextExport(data: Exclude<SpaceNodePayload, { assetPath: string }>): SpaceTextExport {
  const name = safeExportFileName(data.title) || data.kind;
  if (data.kind === "note") return textExport(name, data.plainText);
  if (data.kind === "text") return textExport(name, data.plainText);
  if (data.kind === "markdown") return markdownExport(name, formatExportMarkdown(data.title, data.text));
  if (data.kind === "website") return markdownExport(name, websiteMarkdown(data));
  if (data.kind === "subtitle") return subtitleExport(name, data);
  if (data.kind === "credential") return jsonExport(name, credentialMetadata(data));
  if (data.kind === "object") return jsonExport(name, objectDocument(data));
  if (data.kind === "portal") return jsonExport(name, portalDocument(data));
  data satisfies never;
  throw new Error("Unsupported node export");
}

function markdownExport(name: string, text: string): SpaceTextExport {
  return { defaultPath: `${name}.md`, filters: [{ name: "Markdown", extensions: ["md"] }], text };
}

function textExport(name: string, text: string): SpaceTextExport {
  return { defaultPath: `${name}.txt`, filters: [{ name: "Text", extensions: ["txt"] }], text };
}

function jsonExport(name: string, value: unknown): SpaceTextExport {
  return {
    defaultPath: `${name}.json`,
    filters: [{ name: "JSON", extensions: ["json"] }],
    text: `${JSON.stringify(value, undefined, 2)}\n`,
  };
}

function subtitleExport(name: string, data: SpaceSubtitlePayload): SpaceTextExport {
  const text = data.cues
    .map((cue, index) => `${index + 1}\n${srtTime(cue.startMs)} --> ${srtTime(cue.endMs)}\n${cue.text}`)
    .join("\n\n");
  return {
    defaultPath: `${name}.srt`,
    filters: [{ name: "SubRip subtitles", extensions: ["srt"] }],
    text: `${text}\n`,
  };
}

function srtTime(milliseconds: number) {
  const hours = Math.floor(milliseconds / 3_600_000);
  const minutes = Math.floor((milliseconds % 3_600_000) / 60_000);
  const seconds = Math.floor((milliseconds % 60_000) / 1_000);
  const millis = milliseconds % 1_000;
  return `${pad(hours, 2)}:${pad(minutes, 2)}:${pad(seconds, 2)},${pad(millis, 3)}`;
}

function pad(value: number, length: number) {
  return String(value).padStart(length, "0");
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

function credentialMetadata(data: Extract<SpaceNodePayload, { kind: "credential" }>) {
  return {
    kind: data.kind,
    title: data.title,
    credentialKind: data.credentialKind,
    configured: data.credentialConfigured === true,
  };
}

function objectDocument(data: Extract<SpaceNodePayload, { kind: "object" }>) {
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

function portalDocument(data: Extract<SpaceNodePayload, { kind: "portal" }>) {
  return {
    kind: data.kind,
    title: data.title,
    targetSpaceId: data.targetSpaceId,
    targetSceneId: data.targetSceneId,
    targetNodeId: data.targetNodeId,
    targetStructureNodeId: data.targetStructureNodeId,
  };
}

function sectionOutlineMembers(
  parentId: string,
  index: StructureIndex,
  nodesById: ReadonlyMap<string, SpaceCanvasNode>,
): unknown[] {
  return (index.childrenByParent.get(parentId) ?? []).map((member) => {
    if (member.kind !== "content")
      return {
        kind: member.kind,
        title: member.title,
        members: sectionOutlineMembers(member.id, index, nodesById),
      };
    const node = nodesById.get(member.nodeId);
    if (!node) throw new Error(`Section content node not found: ${member.nodeId}`);
    return { kind: node.data.kind, title: node.data.title, content: safeOutlineContent(node.data) };
  });
}

function safeOutlineContent(data: SpaceNodePayload): unknown {
  if (isSpaceManagedAssetPayload(data))
    return { assetType: data.kind, fileName: data.fileName, size: data.size };
  if (data.kind === "note" || data.kind === "text") return { plainText: data.plainText };
  if (data.kind === "markdown") return { text: data.text };
  if (data.kind === "website")
    return {
      url: data.url,
      canonicalUrl: data.canonicalUrl,
      description: data.description,
      contentPreview: data.contentPreview,
      sections: data.websiteSections,
    };
  if (data.kind === "subtitle")
    return { language: data.language, durationMs: data.durationMs, cues: data.cues };
  if (data.kind === "credential") return credentialMetadata(data);
  if (data.kind === "object") return objectDocument(data);
  if (data.kind === "portal") return portalDocument(data);
  data satisfies never;
  throw new Error("Unsupported section member");
}
