import { convertFileSrc } from "@tauri-apps/api/core";
import { isString } from "es-toolkit";

import { packageAssetPrefix } from "../contracts/generated/contract-values";
import type { TiptapJsonNode } from "../editor/tiptap-document";
import { isSpaceNotePayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";

type DuplicateAsset = (path: string) => Promise<string>;

export async function duplicateTiptapAssets(
  data: SpaceNodePayload,
  duplicate: DuplicateAsset,
): Promise<string[]> {
  if (!isSpaceNotePayload(data)) return [];
  const created: string[] = [];
  const copies = new Map<string, string>();
  for (const node of imageNodes(data.tiptapContent)) {
    const source = imageAssetPath(node);
    if (!source) continue;
    let copy = copies.get(source);
    if (!copy) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Copies are sequenced so repeated sources reuse the first completed copy.
      copy = await duplicate(source);
      copies.set(source, copy);
      created.push(copy);
    }
    const attrs = (node.attrs ??= {});
    attrs.assetPath = copy;
    attrs.src = convertFileSrc(copy);
  }
  return created;
}

function imageNodes(root: TiptapJsonNode) {
  const images: TiptapJsonNode[] = [];
  const visit = (node: TiptapJsonNode) => {
    if (node.type === "image") images.push(node);
    for (const child of node.content ?? []) visit(child);
  };
  visit(root);
  return images;
}

function imageAssetPath(node: TiptapJsonNode) {
  const canonical = node.attrs?.assetPath;
  return isString(canonical) && canonical.length > 0 && !canonical.startsWith(packageAssetPrefix)
    ? canonical
    : undefined;
}
