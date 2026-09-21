import { invoke } from "@tauri-apps/api/core";
import { compact, isString } from "es-toolkit";

import type { PositionedSpaceNode, SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceSectionPayload } from "../model/nodes/capabilities";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { createSpaceClipboardPayload } from "./payload";
import type { SpaceClipboardPayload } from "./payload-types";

const HTML_MARKER = /<!--knovum-spaces:([^>]*)-->/u;

export function writeSpaceClipboard(event: ClipboardEvent, nodes: readonly PositionedSpaceNode[]) {
  if (!event.clipboardData || nodes.length === 0) return false;
  const payload = createSpaceClipboardPayload(nodes);
  event.clipboardData.setData("text/plain", clipboardText(nodes.map(({ node }) => node)));
  event.clipboardData.setData("text/html", clipboardHtml(payload));
  return true;
}

export async function writeSpaceClipboardDirect(nodes: readonly PositionedSpaceNode[]) {
  if (!nodes.length) return false;
  await invoke<void>("plugin:clipboard|write_html_and_text", {
    html: clipboardHtml(createSpaceClipboardPayload(nodes)),
    text: clipboardText(nodes.map(({ node }) => node)),
  });
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
  if (value.length > 8 * 1024 * 1024) return undefined;
  try {
    const parsed = JSON.parse(value) as Partial<SpaceClipboardPayload>;
    if (parsed.version !== 1 || !Array.isArray(parsed.nodes) || !validNodeCount(parsed.nodes.length))
      return undefined;
    const nodes = parsed.nodes.map((node) => {
      if (!isString(node?.id) || !node.id || node.id.length > 128 || !validPosition(node.position)) {
        throw new Error("Invalid clipboard node");
      }
      return {
        id: node.id,
        position: { x: node.position.x, y: node.position.y },
        data: validateSpaceNodePayload(node.data),
      };
    });
    return { version: 1, nodes };
  } catch {
    return undefined;
  }
}

function validNodeCount(count: number) {
  return count > 0 && count <= 200;
}

function validPosition(position: { x?: number; y?: number } | undefined) {
  return Boolean(position && Number.isFinite(position.x) && Number.isFinite(position.y));
}

function clipboardText(nodes: readonly SpaceCanvasNode[]) {
  return compact(
    nodes
      .filter((node) => !isSpaceSectionPayload(node.data))
      .map(({ data }) => {
        if (data.kind === "website") return data.url ?? data.title;
        if ((data.kind === "video" || data.kind === "audio") && data.origin) return data.origin.url;
        if (data.kind === "note" || data.kind === "text") return data.plainText ?? data.title;
        if (data.kind === "markdown") return data.text ?? data.title;
        return data.title;
      }),
  ).join("\n\n");
}
