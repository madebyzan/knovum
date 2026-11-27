import { renderMermaidToPngBase64 } from "../../../../shared/ui/mermaid-export";
import type { TiptapJsonNode } from "../../document-editing/tiptap-document";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";

const VISUAL_EXPORT_FORMATS = new Set(["html", "docx", "pdf", "pptx"]);
const FENCE_OPEN_PATTERN = /^[ \t]{0,3}(`{3,}|~{3,})[ \t]*([^\s`~]*)/u;

function parseFenceOpen(line: string) {
  const match = FENCE_OPEN_PATTERN.exec(line);
  if (!match) return null;
  const marker = match[1] ?? "";
  return {
    char: marker[0] ?? "",
    len: marker.length,
    lang: (match[2] ?? "").toLowerCase(),
  };
}

function isFenceClose(line: string, char: string, len: number): boolean {
  const trimmed = line.trim();
  return trimmed.length >= len && [...trimmed].every((ch) => ch === char);
}

export function extractMarkdownMermaidBlocks(markdown: string): string[] {
  const blocks: string[] = [];
  let fence: { char: string; len: number; isMermaid: boolean } | null = null;
  let buffer: string[] = [];

  for (const line of markdown.split(/\r?\n/u)) {
    if (!fence) {
      const opened = parseFenceOpen(line);
      if (opened) {
        fence = { char: opened.char, len: opened.len, isMermaid: opened.lang === "mermaid" };
        buffer = [];
      }
      continue;
    }
    if (isFenceClose(line, fence.char, fence.len)) {
      const source = buffer.join("\n").trim();
      if (fence.isMermaid && source) blocks.push(source);
      fence = null;
      buffer = [];
      continue;
    }
    if (fence.isMermaid) buffer.push(line);
  }
  return blocks;
}

function collectTiptapText(node: TiptapJsonNode): string {
  if (typeof node.text === "string") return node.text;
  return (node.content ?? []).map(collectTiptapText).join("");
}

export function extractTiptapMermaidBlocks(root: TiptapJsonNode): string[] {
  const blocks: string[] = [];
  const visit = (node: TiptapJsonNode) => {
    if (
      node.type === "codeBlock" &&
      typeof node.attrs?.language === "string" &&
      node.attrs.language.toLowerCase() === "mermaid"
    ) {
      const source = collectTiptapText(node).trim();
      if (source) blocks.push(source);
      return;
    }
    for (const child of node.content ?? []) visit(child);
  };
  visit(root);
  return blocks;
}

function extractNodeMermaidBlocks(data: SpaceNodePayload): string[] {
  if (data.kind === "markdown") return extractMarkdownMermaidBlocks(data.text);
  if (data.kind === "note" || data.kind === "text") return extractTiptapMermaidBlocks(data.tiptapContent);
  return [];
}

async function diagramKey(source: string): Promise<string> {
  const bytes = new TextEncoder().encode(source.trim());
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  const hex = Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
  return `mermaid:${hex}`;
}

/**
 * Renders Mermaid code blocks in a document to 2x Retina base64 PNGs keyed by `mermaid:<sha256>`
 * when exporting to visual formats (HTML, DOCX, PDF, PPTX). Markdown and TXT exports keep source.
 */
export async function renderNodeMermaidDiagrams(
  data: SpaceNodePayload,
  format: string,
): Promise<Record<string, string> | undefined> {
  if (!VISUAL_EXPORT_FORMATS.has(format)) return undefined;
  const uniqueSources = [...new Set(extractNodeMermaidBlocks(data))];
  if (!uniqueSources.length) return undefined;

  const rendered: Record<string, string> = {};
  for (const source of uniqueSources) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Mermaid shares one global render configuration per call.
    const pngBase64 = await renderMermaidToPngBase64(source);
    if (!pngBase64) continue;
    // oxlint-disable-next-line eslint/no-await-in-loop -- Compute key sequentially alongside diagram rendering.
    rendered[await diagramKey(source)] = pngBase64;
  }
  return Object.keys(rendered).length ? rendered : undefined;
}
