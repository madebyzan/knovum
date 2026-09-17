import { isPlainObject } from "es-toolkit";

import { restorePackagedNodeAssets } from "../assets/managed-assets";
import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { remapNodePayload } from "./content-remap";
import { requiredMap } from "./id-maps";
import type { SpacePackageRestoreContext } from "./restore-context";
import type { SpacePackageRestoreRequest } from "./restore-transport";

export function restorePackageNodes(
  context: SpacePackageRestoreContext,
): SpacePackageRestoreRequest["nodes"] {
  return context.manifest.nodes.map((node) => ({
    id: requiredMap(context.maps.nodes, node.id, "node"),
    kind: node.kind,
    data: restoreNodeData(context, node.data),
    createdAt: node.createdAt,
    updatedAt: node.updatedAt,
  }));
}

function restoreNodeData(context: SpacePackageRestoreContext, value: unknown): SpaceNodePayload {
  if (!isPlainObject(value)) {
    throw new Error("Package node data is invalid");
  }
  const raw = structuredClone(value) as Record<string, unknown>;
  restorePackagedNodeAssets(raw, context.assets);
  if (!isPlainObject(raw.origin)) delete raw.previewPath;
  if (raw.kind === "credential") raw.credentialConfigured = false;
  const data = validateSpaceNodePayload(raw);
  const remapped = remapNodePayload(data, context.maps, context.manifest.objectTypes);
  return prepareSpaceKnowledgePayload(validateSpaceNodePayload(remapped));
}
