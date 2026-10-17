import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { StructureIndex } from "../../model/structure/structure-index";
import { validateStructureSnapshot } from "../../model/structure/structure-validation";
import { exportFilter, safeExportFileName } from "./naming";
import { credentialMetadata, json, objectDocument, portalDocument } from "./payloads";
import { saveSpaceTextFile } from "./text";

/**
 * A section is an arrangement rather than a resource, so it is exported as the
 * outline of what it holds instead of as one of its members' formats.
 */
export async function exportSpaceSection(
  snapshot: SpaceEditSnapshot,
  nodes: readonly SpaceCanvasNode[],
  sectionId: string,
): Promise<boolean> {
  const index = validateStructureSnapshot(snapshot.structure, snapshot.layouts);
  const section = index.structureById.get(sectionId);
  if (section?.kind !== "section") throw new Error(`Section not found: ${sectionId}`);
  const nodesById = new Map(nodes.map((node) => [node.id, node]));
  const outline = json(() => ({
    version: 1,
    kind: "section",
    title: section.title,
    members: sectionOutlineMembers(sectionId, index, nodesById),
  }));
  const name = safeExportFileName(section.title) || "section";
  return saveSpaceTextFile({
    defaultPath: `${name}.${outline.extension}`,
    filters: [exportFilter(outline.extension)],
    text: outline.text(),
  });
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
  if (data.kind === "credential") return credentialMetadata(data);
  if (data.kind === "object") return objectDocument(data);
  if (data.kind === "portal") return portalDocument(data);
  data satisfies never;
  throw new Error("Unsupported section member");
}
