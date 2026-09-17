import { convertFileSrc } from "@tauri-apps/api/core";
import { isPlainObject, isString } from "es-toolkit";

import type { TiptapJsonNode } from "../editor/tiptap-document";
import { isSpaceNotePayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { PACKAGE_ASSET_PREFIX } from "../package/contract";

const TOP_LEVEL_ASSET_FIELDS = ["assetPath", "faviconPath", "previewImagePath"] as const;

type DuplicateAsset = (path: string) => Promise<string>;

export function restorePackagedNodeAssets(data: Record<string, unknown>, assets: Map<string, string>) {
  for (const field of TOP_LEVEL_ASSET_FIELDS) {
    const value = data[field];
    if (isString(value) && value.startsWith(PACKAGE_ASSET_PREFIX)) {
      data[field] = requiredAsset(assets, value);
    }
  }
  if (isPlainObject(data.origin)) restoreAssetField(data, "previewPath", assets);
  if (isTiptapNode(data.tiptapContent)) restoreTiptapAssets(data.tiptapContent, assets);
}

function restoreAssetField(data: Record<string, unknown>, field: string, assets: Map<string, string>) {
  const value = data[field];
  if (isString(value) && value.startsWith(PACKAGE_ASSET_PREFIX)) {
    data[field] = requiredAsset(assets, value);
  }
}

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

export function tiptapAssetPaths(content: TiptapJsonNode | undefined) {
  const paths = new Set<string>();
  if (!content) return paths;
  for (const node of imageNodes(content)) {
    const path = imageAssetPath(node);
    if (path) paths.add(path);
  }
  return paths;
}

function restoreTiptapAssets(content: TiptapJsonNode, assets: Map<string, string>) {
  for (const node of imageNodes(content)) {
    const attrs = node.attrs;
    if (!attrs) continue;
    const packaged = [attrs.assetPath, attrs.src].find(
      (value): value is string => isString(value) && value.startsWith(PACKAGE_ASSET_PREFIX),
    );
    if (!packaged) continue;
    const path = requiredAsset(assets, packaged);
    attrs.assetPath = path;
    attrs.src = convertFileSrc(path);
  }
}

function requiredAsset(assets: Map<string, string>, packaged: string) {
  const token = packaged.slice(PACKAGE_ASSET_PREFIX.length);
  const path = assets.get(token);
  if (!path) throw new Error("Package references an unknown asset");
  return path;
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
  return isString(canonical) && canonical.length > 0 && !canonical.startsWith(PACKAGE_ASSET_PREFIX)
    ? canonical
    : undefined;
}

function isTiptapNode(value: unknown): value is TiptapJsonNode {
  return isPlainObject(value) && "type" in value;
}
