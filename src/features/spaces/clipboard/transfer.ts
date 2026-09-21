import { compact, isPlainObject } from "es-toolkit";
import { writeHtmlAndText } from "tauri-plugin-clipboard-api";

import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { structureLayoutSchema, structureNodeSchema } from "../model/structure/structure-contract";
import { validateStructureSnapshot } from "../model/structure/structure-validation";
import type { SpaceClipboardPayload } from "./payload-types";

const HTML_MARKER = /<!--knovum-spaces:([^>]*)-->/u;

export async function writeSpaceClipboardDirect(payload: SpaceClipboardPayload) {
  if (!payload.structure.length) return false;
  await writeHtmlAndText(clipboardHtml(payload), clipboardText(payload));
  return true;
}

export function readSpaceClipboard(event: ClipboardEvent): SpaceClipboardPayload | undefined {
  return readSpaceClipboardHtml(event.clipboardData?.getData("text/html") ?? "");
}

export function readSpaceClipboardHtml(html: string): SpaceClipboardPayload | undefined {
  const marker = html.match(HTML_MARKER)?.[1];
  if (!marker) return undefined;
  try {
    return parsePayload(decodeURIComponent(marker));
  } catch {
    return undefined;
  }
}

function clipboardHtml(payload: SpaceClipboardPayload) {
  return `<!--knovum-spaces:${encodeURIComponent(JSON.stringify(payload))}-->`;
}

function parsePayload(value: string): SpaceClipboardPayload | undefined {
  if (value.length > 8 * 1024 * 1024) return undefined;
  const source: unknown = JSON.parse(value);
  if (!isClipboardEnvelope(source)) return undefined;
  if (!validCount(source.structure.length) || source.layouts.length !== source.structure.length)
    return undefined;
  const structure = source.structure.map((structureNode) => structureNodeSchema.parse(structureNode));
  const layouts = source.layouts.map((layout) => structureLayoutSchema.parse(layout));
  validateStructureSnapshot(structure, layouts);
  const nodes = source.nodes.map(parseClipboardNode);
  const nodeIds = new Set(nodes.map(({ id }) => id));
  const expected = new Set(
    structure.flatMap((structureNode) => (structureNode.kind === "content" ? [structureNode.nodeId] : [])),
  );
  validateClipboardIdentities(nodeIds, nodes.length, expected);
  const relations = source.relations as SpaceClipboardPayload["relations"];
  validateClipboardRelations(relations, nodeIds);
  return { version: 1, structure, layouts, nodes, relations };
}

function isClipboardEnvelope(source: unknown): source is Record<string, unknown> & {
  version: 1;
  structure: unknown[];
  layouts: unknown[];
  nodes: unknown[];
  relations: unknown[];
} {
  return Boolean(
    isPlainObject(source) &&
    source.version === 1 &&
    Array.isArray(source.structure) &&
    Array.isArray(source.layouts) &&
    Array.isArray(source.nodes) &&
    Array.isArray(source.relations),
  );
}

function parseClipboardNode(node: unknown) {
  if (!isPlainObject(node) || typeof node.id !== "string" || !node.id)
    throw new Error("Invalid clipboard node");
  return { id: node.id, data: validateSpaceNodePayload(node.data) };
}

function validateClipboardIdentities(
  nodeIds: ReadonlySet<string>,
  count: number,
  expected: ReadonlySet<string>,
) {
  if (
    nodeIds.size !== count ||
    nodeIds.size !== expected.size ||
    [...nodeIds].some((id) => !expected.has(id))
  )
    throw new Error("Clipboard content identity mismatch");
}

function validateClipboardRelations(
  relations: SpaceClipboardPayload["relations"],
  nodeIds: ReadonlySet<string>,
) {
  const invalid = relations.some(
    (relation) =>
      !isPlainObject(relation) ||
      typeof relation.id !== "string" ||
      !nodeIds.has(relation.sourceId) ||
      !nodeIds.has(relation.targetId),
  );
  if (invalid) throw new Error("Invalid clipboard relation");
}

function validCount(count: number) {
  return count > 0 && count <= 100_000;
}

function clipboardText(payload: SpaceClipboardPayload) {
  return compact(payload.nodes.map(({ data }) => clipboardNodeText(data))).join("\n\n");
}

function clipboardNodeText(data: SpaceClipboardPayload["nodes"][number]["data"]) {
  if (data.kind === "website") return data.url ?? data.title;
  return mediaOriginUrl(data) ?? editableText(data) ?? data.title;
}

function mediaOriginUrl(data: SpaceClipboardPayload["nodes"][number]["data"]) {
  if (data.kind !== "video" && data.kind !== "audio") return undefined;
  return data.origin?.url;
}

function editableText(data: SpaceClipboardPayload["nodes"][number]["data"]) {
  if (data.kind === "note" || data.kind === "text") return data.plainText || undefined;
  return data.kind === "markdown" ? data.text || undefined : undefined;
}
