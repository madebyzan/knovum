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
  // Content the engine can read or assemble is serialized by the engine: managed assets, the
  // editable documents a Note and a text block hold, and the summary of a capture.
  if (
    isSpaceManagedAssetPayload(data) ||
    data.kind === "note" ||
    data.kind === "text" ||
    data.kind === "website"
  )
    return [];
  if (data.kind === "markdown")
    return [{ extension: "md", text: () => formatExportMarkdown(data.title, data.text) }];
  if (data.kind === "credential") return [json(() => credentialMetadata(data))];
  if (data.kind === "object") return [json(() => objectDocument(data))];
  if (data.kind === "portal") return [json(() => portalDocument(data))];
  data satisfies never;
  throw new Error("Unsupported node export");
}

export function json(value: () => unknown): PayloadExport {
  return { extension: "json", text: () => `${JSON.stringify(value(), undefined, 2)}\n` };
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
