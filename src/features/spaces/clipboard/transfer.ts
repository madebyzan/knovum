import { writeHtmlAndText } from "tauri-plugin-clipboard-api";

import { parseSpaceClipboardPayload } from "./payload-schema";
import type { SpaceClipboardPayload } from "./payload-types";

const MARKER = /<!--knovum-spaces:([^>]*)-->/u;
const MAX_BYTES = 8 * 1024 * 1024;
export async function writeSpaceClipboardDirect(payload: SpaceClipboardPayload) {
  await writeHtmlAndText(clipboardHtml(payload), clipboardText(payload));
}
export function writeSpaceClipboard(event: ClipboardEvent, payload: SpaceClipboardPayload) {
  if (!event.clipboardData) return false;
  event.clipboardData.setData("text/html", clipboardHtml(payload));
  event.clipboardData.setData("text/plain", clipboardText(payload));
  return true;
}
export function readSpaceClipboard(event: ClipboardEvent) {
  return readSpaceClipboardHtml(event.clipboardData?.getData("text/html") ?? "");
}
export function readSpaceClipboardHtml(html: string): SpaceClipboardPayload | undefined {
  const encoded = html.match(MARKER)?.[1];
  if (encoded === undefined) return undefined;
  if (encoded.length > MAX_BYTES * 3) throw new Error("Native clipboard content is too large");
  const source = decodeURIComponent(encoded);
  if (new TextEncoder().encode(source).byteLength > MAX_BYTES)
    throw new Error("Native clipboard content is too large");
  // A malformed native payload fails explicitly; it is not silently reinterpreted as plain text.
  return parseSpaceClipboardPayload(JSON.parse(source));
}
function clipboardHtml(value: SpaceClipboardPayload) {
  const source = JSON.stringify(parseSpaceClipboardPayload(value));
  if (new TextEncoder().encode(source).byteLength > MAX_BYTES)
    throw new Error("Selection exceeds the 8 MiB clipboard limit; export a Space package instead");
  return `<!--knovum-spaces:${encodeURIComponent(source)}-->`;
}
function clipboardText(payload: SpaceClipboardPayload) {
  const parts = payload.nodes
    .map(({ data }) => {
      if (data.kind === "website") return data.url;
      if ((data.kind === "video" || data.kind === "audio") && data.origin) return data.origin.url;
      if (data.kind === "note" || data.kind === "text") return data.plainText;
      if (data.kind === "markdown") return data.text;
      return data.title;
    })
    .filter(Boolean);
  return parts.length
    ? parts.join("\n\n")
    : payload.structure.items.flatMap((item) => (item.kind === "content" ? [] : [item.title])).join("\n");
}
