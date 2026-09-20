import { duplicateSpaceAsset, removeSpaceAssets } from "../assets/asset-operations";
import type { EditorFragment } from "../canvas/contract/editor-fragment";
import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import type { SpaceKnowledgeObjectType } from "../knowledge/model/object-types";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { canonicalRelationEndpoints } from "../model/relations/relation";
import { prepareClipboardTypes } from "./fragment-types";
import { cloneNodeData } from "./node-cloning";
import { parseSpaceClipboardPayload } from "./payload-schema";
import type { SpaceClipboardPayload } from "./payload-types";

export async function prepareClipboardFragment(
  source: SpaceClipboardPayload,
  targetSpaceId: string,
  targetTypes: readonly SpaceKnowledgeObjectType[],
) {
  const payload = parseSpaceClipboardPayload(source);
  const identities = new Map(payload.structure.items.map((item) => [item.id, crypto.randomUUID()]));
  const relationshipIds = new Map(payload.relations.map((item) => [item.id, crypto.randomUUID()]));
  const types = prepareClipboardTypes(payload, targetTypes);
  const assets: string[] = [];
  try {
    const context = { identities, assets, sourceSpaceId: payload.sourceSpaceId, targetSpaceId };
    const contents = new Map<string, SpaceNodePayload>();
    for (const node of payload.nodes) {
      // oxlint-disable-next-line no-await-in-loop -- Each staged file has one tracked cleanup owner.
      const data = await cloneNodeData(node.data, context);
      remapObjectReferences(data, payload, identities, types.identities, targetSpaceId);
      contents.set(identities.get(node.id)!, prepareSpaceKnowledgePayload(data));
    }
    for (const [id, data] of contents) {
      if (data.kind !== "subtitle") continue;
      const video = contents.get(data.sourceVideoNodeId);
      if (video?.kind === "video" && video.sourceRevision)
        contents.set(
          id,
          prepareSpaceKnowledgePayload({ ...data, sourceVideoRevision: video.sourceRevision }),
        );
    }
    const layout = {
      version: 1 as const,
      items: payload.layout.items.map((record) => ({
        ...structuredClone(record),
        itemId: identities.get(record.itemId)!,
      })),
    };
    for (const record of layout.items) {
      if (record.kind === "group" || !record.appearance?.background) continue;
      // oxlint-disable-next-line no-await-in-loop -- Track each background before another staged copy can fail.
      const path = await duplicateSpaceAsset(record.appearance.background);
      assets.push(path);
      record.appearance.background = path;
    }
    const fragment: EditorFragment = {
      structure: {
        version: 1,
        items: payload.structure.items.map((item) => ({
          ...item,
          id: identities.get(item.id)!,
          parentId: item.parentId === null ? null : identities.get(item.parentId)!,
          ...(item.kind === "content" ? { nodeId: identities.get(item.nodeId)! } : {}),
        })),
      },
      layout,
      nodes: [...contents].map(([id, data]) => ({ id, data, kind: data.kind, sourceChunks: [] })),
      objectTypes: types.definitions,
      relations: payload.relations.map((relation) => {
        const [sourceId, targetId] = canonicalRelationEndpoints(
          identities.get(relation.sourceId)!,
          identities.get(relation.targetId)!,
          relation.kind,
        );
        return {
          id: relationshipIds.get(relation.id)!,
          sourceId,
          targetId,
          kind: relation.kind,
          rationale: relation.rationale,
          status: relation.status,
          ...(relation.confidence === undefined ? {} : { confidence: relation.confidence }),
        };
      }),
      relationStyles: payload.relations.flatMap((relation) =>
        relation.color ? [{ relationId: relationshipIds.get(relation.id)!, color: relation.color }] : [],
      ),
    };
    return { fragment, assets };
  } catch (cause) {
    await discardClipboardAssets(assets, cause);
    throw cause;
  }
}
function remapObjectReferences(
  data: SpaceNodePayload,
  source: SpaceClipboardPayload,
  identities: ReadonlyMap<string, string>,
  types: ReadonlyMap<string, string>,
  targetSpaceId: string,
) {
  if (data.kind !== "object") return;
  const definition = source.objectTypes.find((type) => type.id === data.objectTypeId);
  data.objectTypeId = types.get(data.objectTypeId) ?? data.objectTypeId;
  for (const field of definition?.fields ?? []) {
    if (field.kind !== "node-ref") continue;
    const value = data.properties[field.key];
    if (typeof value !== "string" || !value) continue;
    if (identities.has(value)) data.properties[field.key] = identities.get(value)!;
    else if (source.sourceSpaceId !== targetSpaceId)
      throw new Error(`Copy the object referenced by '${field.label}' when pasting into another Space`);
  }
}
export async function discardClipboardAssets(paths: readonly string[], original?: unknown) {
  try {
    await removeSpaceAssets([...paths]);
  } catch (cleanup) {
    throw new AggregateError(
      original === undefined ? [cleanup] : [original, cleanup],
      "The edit was rejected and some staged files could not be cleaned up",
    );
  }
}
