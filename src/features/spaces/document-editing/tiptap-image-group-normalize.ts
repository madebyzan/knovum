import type { Node as ProseMirrorNode, NodeType } from "@tiptap/pm/model";
import { Plugin, PluginKey, type EditorState, type Transaction } from "@tiptap/pm/state";

import { isVideoMediaUrl } from "../../../shared/ui/markdown-media";

const IMAGE_GROUP_NORMALIZER_KEY = new PluginKey("spaceImageGroupNormalizer");

interface BlockChild {
  readonly node: ProseMirrorNode;
  readonly pos: number;
}

interface DocReplacement {
  readonly from: number;
  readonly to: number;
  readonly nodes: readonly ProseMirrorNode[];
}

function isAutoImageGroup(node: ProseMirrorNode, groupType: NodeType): boolean {
  return node.type === groupType && (node.attrs.layout ?? "auto") === "auto";
}

function isGroupableImage(node: ProseMirrorNode, imageType: NodeType): boolean {
  if (node.type !== imageType) return false;
  const mediaPath = String(node.attrs.assetPath || node.attrs.src || "");
  return !isVideoMediaUrl(mediaPath);
}

function isMergeableNode(node: ProseMirrorNode, groupType: NodeType, imageType: NodeType): boolean {
  return isGroupableImage(node, imageType) || isAutoImageGroup(node, groupType);
}

function collectRunImages(run: readonly BlockChild[], imageType: NodeType): ProseMirrorNode[] {
  const images: ProseMirrorNode[] = [];
  for (const { node } of run) {
    if (node.type === imageType) {
      images.push(node);
    } else {
      node.forEach((child) => {
        if (child.type === imageType) images.push(child);
      });
    }
  }
  return images;
}

function findRunEnd(
  children: readonly BlockChild[],
  startIndex: number,
  groupType: NodeType,
  imageType: NodeType,
): number {
  let end = startIndex + 1;
  while (end < children.length && isMergeableNode(children[end]!.node, groupType, imageType)) {
    end += 1;
  }
  return end;
}

function resolveRunReplacement(
  run: readonly BlockChild[],
  groupType: NodeType,
  imageType: NodeType,
): DocReplacement | null {
  const images = collectRunImages(run, imageType);
  const from = run[0]!.pos;
  const last = run[run.length - 1]!;
  const to = last.pos + last.node.nodeSize;

  if (images.length === 1) {
    const alreadySingleImage = run.length === 1 && run[0]!.node.type === imageType;
    return alreadySingleImage ? null : { from, to, nodes: [images[0]!] };
  }
  if (images.length >= 2) {
    const alreadyGroup = run.length === 1 && isAutoImageGroup(run[0]!.node, groupType);
    return alreadyGroup ? null : { from, to, nodes: [groupType.create({ layout: "auto" }, images)] };
  }
  return null;
}

function collectDocReplacements(
  doc: ProseMirrorNode,
  groupType: NodeType,
  imageType: NodeType,
): DocReplacement[] {
  const children: BlockChild[] = [];
  doc.forEach((node, offset) => children.push({ node, pos: offset }));

  const replacements: DocReplacement[] = [];
  let index = 0;
  while (index < children.length) {
    const { node, pos } = children[index]!;
    if (node.type === groupType && node.childCount === 0) {
      replacements.push({ from: pos, to: pos + node.nodeSize, nodes: [] });
      index += 1;
      continue;
    }
    if (!isMergeableNode(node, groupType, imageType)) {
      index += 1;
      continue;
    }
    const end = findRunEnd(children, index, groupType, imageType);
    const replacement = resolveRunReplacement(children.slice(index, end), groupType, imageType);
    if (replacement) replacements.push(replacement);
    index = end;
  }
  return replacements;
}

export function normalizeImageGroupsInState(
  state: EditorState,
  groupTypeName: string,
  imageTypeName: string,
): Transaction | null {
  const groupType = state.schema.nodes[groupTypeName];
  const imageType = state.schema.nodes[imageTypeName];
  if (!groupType || !imageType) return null;

  const replacements = collectDocReplacements(state.doc, groupType, imageType);
  if (!replacements.length) return null;

  const tr = state.tr;
  for (let i = replacements.length - 1; i >= 0; i -= 1) {
    const item = replacements[i]!;
    if (item.nodes.length === 0) tr.delete(item.from, item.to);
    else tr.replaceWith(item.from, item.to, item.nodes);
  }
  tr.setMeta("addToHistory", false);
  return tr.docChanged ? tr : null;
}

export function createImageGroupNormalizerPlugin(groupTypeName: string, imageTypeName: string): Plugin {
  return new Plugin({
    key: IMAGE_GROUP_NORMALIZER_KEY,
    appendTransaction(transactions, _oldState, newState) {
      if (!transactions.some((tr) => tr.docChanged)) return null;
      return normalizeImageGroupsInState(newState, groupTypeName, imageTypeName);
    },
  });
}
