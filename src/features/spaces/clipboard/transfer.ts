import { compact } from "es-toolkit";
import { writeHtmlAndText } from "tauri-plugin-clipboard-api";
import { z } from "zod";

import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { toSpaceRelation } from "../model/relations/schema/relation";
import {
  MAX_STRUCTURE_NODES,
  structureLayoutSchema,
  structureNodeSchema,
} from "../model/structure/schema/structure";
import { validateStructureSnapshot } from "../model/structure/structure-validation";
import type { SpaceClipboardPayload } from "./clipboard-envelope";
import { clipboardRelationSchema } from "./schema/relation";

const HTML_MARKER = /<!--knovum-spaces:([^>]*)-->/u;
const MAX_CLIPBOARD_BYTES = 8 * 1024 * 1024;

const clipboardNodeSchema = z.object({ id: z.string().min(1), data: z.unknown() }).strict();

const clipboardEnvelopeSchema = z
  .object({
    version: z.literal(1),
    structure: z.array(structureNodeSchema).min(1).max(MAX_STRUCTURE_NODES),
    layouts: z.array(structureLayoutSchema).max(MAX_STRUCTURE_NODES),
    nodes: z.array(clipboardNodeSchema).max(MAX_STRUCTURE_NODES),
    relations: z.array(clipboardRelationSchema).max(MAX_STRUCTURE_NODES),
  })
  .strict()
  .refine(({ structure, layouts }) => structure.length === layouts.length, {
    message: "Clipboard structure and layout counts differ",
    path: ["layouts"],
  });

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
  if (value.length > MAX_CLIPBOARD_BYTES) return undefined;
  const envelope = clipboardEnvelopeSchema.safeParse(JSON.parse(value));
  if (!envelope.success) return undefined;
  const { structure, layouts, nodes: copiedNodes, relations } = envelope.data;
  validateStructureSnapshot(structure, layouts);
  const nodes = copiedNodes.map(({ id, data }) => ({ id, data: validateSpaceNodePayload(data) }));
  const nodeIds = new Set(nodes.map(({ id }) => id));
  const expected = new Set(
    structure.flatMap((structureNode) => (structureNode.kind === "content" ? [structureNode.nodeId] : [])),
  );
  validateClipboardIdentities(nodeIds, nodes.length, expected);
  validateClipboardRelations(relations, nodeIds);
  return { version: 1, structure, layouts, nodes, relations: relations.map(toSpaceRelation) };
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
  relations: readonly z.infer<typeof clipboardRelationSchema>[],
  nodeIds: ReadonlySet<string>,
) {
  const detached = relations.some(
    ({ sourceId, targetId }) => !nodeIds.has(sourceId) || !nodeIds.has(targetId),
  );
  if (detached) throw new Error("Clipboard relation endpoints are outside the copied content");
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
