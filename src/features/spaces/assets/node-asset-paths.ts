import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import {
  isSpaceManagedAssetPayload,
  isSpaceMarkdownPayload,
  isSpaceNotePayload,
  isSpaceWebsitePayload,
} from "../model/nodes/capabilities";
import { tiptapAssetPaths } from "./managed-assets";

export function assetPathsForNodes(nodes: readonly SpaceCanvasNode[]) {
  return nodes.flatMap(({ data }) => {
    const paths = topLevelAssetPaths(data);
    if (isSpaceNotePayload(data)) paths.push(...tiptapAssetPaths(data.tiptapContent));
    if (isSpaceMarkdownPayload(data)) {
      if (data.tiptapContent) paths.push(...tiptapAssetPaths(data.tiptapContent));
      if (data.text) paths.push(...markdownAssetPaths(data.text));
    }
    return paths;
  });
}

function topLevelAssetPaths(data: SpaceCanvasNode["data"]): string[] {
  if (isSpaceManagedAssetPayload(data)) {
    const preview =
      (data.kind === "video" || data.kind === "audio") && data.origin ? data.previewPath : undefined;
    return [data.assetPath, preview].filter((path): path is string => path !== undefined);
  }
  if (!isSpaceWebsitePayload(data)) return [];
  return [data.faviconPath, data.previewImagePath].filter((path): path is string => path !== undefined);
}

function markdownAssetPaths(text: string): string[] {
  const paths: string[] = [];
  const imageRegex = /!\[[^\]]*\]\(([^)\s]+)[^)]*\)/gu;
  for (const match of text.matchAll(imageRegex)) {
    const raw = match[1];
    if (!raw || raw.startsWith("knovum-package-asset:")) continue;
    paths.push(extractPathFromUrl(raw));
  }
  return paths;
}

function extractPathFromUrl(url: string): string {
  if (url.startsWith("asset://localhost/")) {
    return decodeURIComponent(url.slice("asset://localhost/".length));
  }
  if (url.startsWith("http://asset.localhost/")) {
    return decodeURIComponent(url.slice("http://asset.localhost/".length));
  }
  if (url.startsWith("asset://")) {
    return decodeURIComponent(url.slice("asset://".length));
  }
  return url;
}
