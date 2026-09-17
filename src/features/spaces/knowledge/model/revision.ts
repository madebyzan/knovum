import { isBoolean, isNumber, isPlainObject, isString } from "es-toolkit";

import type { TiptapJsonNode } from "../../editor/tiptap-document";
import { isSpaceManagedAssetPayload, type SpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { compareUtf8Binary } from "../../model/text";
import { isSpaceSemanticNodeData } from "./semantic-node";

const REVISION_PREFIX = "node1:";
const encoder = new TextEncoder();
const LOCAL_TIPTAP_ATTRS = new Set(["assetPath", "previewPath", "src"]);

export function knowledgeRevision(data: SpaceNodePayload): string | undefined {
  if (!isSpaceSemanticNodeData(data)) return undefined;
  const payload = stableJson({
    kind: data.kind,
    title: data.title,
    tags: data.tags,
    ...revisionContent(data),
  });
  let hash = 0xcbf29ce484222325n;
  for (const byte of encoder.encode(payload)) {
    hash ^= BigInt(byte);
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return `${REVISION_PREFIX}${hash.toString(16).padStart(16, "0")}`;
}

function revisionContent(data: SpaceNodePayload): Record<string, unknown> {
  if (data.kind === "note") {
    return { tiptapContent: semanticTiptap(data.tiptapContent), plainText: data.plainText };
  }
  if (data.kind === "markdown") return { text: data.text };
  if (data.kind === "subtitle") {
    return {
      sourceVideoNodeId: data.sourceVideoNodeId,
      sourceVideoRevision: data.sourceVideoRevision,
      language: data.language,
      durationMs: data.durationMs,
      cues: data.cues,
    };
  }
  if (data.kind === "website") return websiteRevision(data);
  if (data.kind === "object") return objectRevision(data);
  if (isSpaceManagedAssetPayload(data)) return managedAssetRevision(data);
  return {};
}

function websiteRevision(data: Extract<SpaceNodePayload, { kind: "website" }>) {
  return {
    url: data.url,
    siteName: data.siteName,
    description: data.description,
    contentPreview: data.contentPreview,
    websiteSections: data.websiteSections,
  };
}

function objectRevision(data: Extract<SpaceNodePayload, { kind: "object" }>) {
  return {
    objectTypeId: data.objectTypeId,
    summary: data.summary,
    status: data.status,
    properties: data.properties,
  };
}

function managedAssetRevision(data: SpaceManagedAssetPayload) {
  const media = data.kind === "video" || data.kind === "audio";
  const visual = data.kind === "image" || data.kind === "video";
  return {
    fileName: data.fileName,
    previewText: data.previewText,
    ...(media
      ? {
          origin: data.origin,
          creator: data.creator,
          durationSeconds: data.durationSeconds,
          sourceRevision: data.sourceRevision,
        }
      : {}),
    ...(visual ? { visualKnowledge: data.visualKnowledge } : {}),
    ...(data.kind === "file" && data.documentCoverage
      ? {
          documentKnowledge: data.documentCoverage.occurrences
            .filter((occurrence) => occurrence.analysis !== null)
            .map(({ id, location, analysis }) => ({ id, location, analysis })),
        }
      : {}),
  };
}

function semanticTiptap(value: TiptapJsonNode): unknown {
  return sanitizeValue(value);
}

function sanitizeValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sanitizeValue);
  if (!isPlainObject(value)) return value;
  const object = value as Record<string, unknown>;
  const imageNode = object.type === "image";
  const sanitized: Record<string, unknown> = {};
  for (const key of Object.keys(object).toSorted(compareUtf8Binary)) {
    const child = object[key];
    if (child === undefined) continue;
    sanitized[key] = key === "attrs" ? sanitizeAttrs(child, imageNode) : sanitizeValue(child);
  }
  return sanitized;
}

function sanitizeAttrs(value: unknown, imageNode: boolean): unknown {
  if (!isPlainObject(value)) return sanitizeValue(value);
  const attrs = value as Record<string, unknown>;
  const managedImage = imageNode && isString(attrs.assetPath) && attrs.assetPath.length > 0;
  const sanitized: Record<string, unknown> = {};
  for (const key of Object.keys(attrs).toSorted(compareUtf8Binary)) {
    if (managedImage && LOCAL_TIPTAP_ATTRS.has(key)) continue;
    const child = attrs[key];
    if (child !== undefined) sanitized[key] = sanitizeValue(child);
  }
  return sanitized;
}

function canonicalNumber(value: number): string {
  if (!Number.isFinite(value)) throw new Error("Knowledge revision numbers must be finite");
  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);
  view.setFloat64(0, Object.is(value, -0) ? 0 : value, false);
  return `n${view.getBigUint64(0, false).toString(16).padStart(16, "0")}`;
}

function stableJson(value: unknown): string {
  if (value === null) return "null";
  if (isString(value) || isBoolean(value)) return JSON.stringify(value);
  if (isNumber(value)) return canonicalNumber(value);
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (isPlainObject(value)) {
    const fields = Object.entries(value as Record<string, unknown>)
      .filter(([, item]) => item !== undefined)
      .toSorted(([left], [right]) => compareUtf8Binary(left, right))
      .map(([key, item]) => `${JSON.stringify(key)}:${stableJson(item)}`);
    return `{${fields.join(",")}}`;
  }
  return "null";
}
