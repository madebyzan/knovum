import type { KnowledgeLocator } from "../knowledge/model/schema/chunk";

interface ProseMirrorNode {
  type: { name: string };
  attrs: Record<string, unknown>;
  textContent: string;
  childCount: number;
  child(index: number): ProseMirrorNode;
  forEach(visitor: (node: ProseMirrorNode, offset: number, index: number) => void): void;
  descendants(visitor: (node: ProseMirrorNode, position: number) => boolean | void): void;
}

interface LocatableEditor {
  state: { doc: ProseMirrorNode };
  commands: { setTextSelection(position: number): boolean; scrollIntoView(): boolean };
}

export function revealKnowledgeInEditor(editor: LocatableEditor | undefined, locator: KnowledgeLocator) {
  if (!editor) return false;
  const position = locate(editor.state.doc, locator);
  if (position === undefined) return false;
  editor.commands.setTextSelection(Math.max(1, position));
  editor.commands.scrollIntoView();
  return true;
}

function locate(doc: ProseMirrorNode, locator: KnowledgeLocator) {
  const note = locateNoteBlock(doc, locator);
  if (note !== undefined) return note;
  const heading = locateMarkdownHeading(doc, locator);
  if (heading !== undefined) return heading;
  return locateQuotedText(doc, locator);
}

function locateNoteBlock(doc: ProseMirrorNode, locator: KnowledgeLocator) {
  const location = locator.location;
  if (location.kind !== "note" || !location.blockId?.startsWith("block-")) return undefined;
  const index = Number(location.blockId.slice(6));
  if (!Number.isInteger(index) || index < 0) return undefined;
  let match: number | undefined;
  doc.forEach((_node, offset, childIndex) => {
    if (childIndex === index) match = offset + 1;
  });
  return match;
}

function locateMarkdownHeading(doc: ProseMirrorNode, locator: KnowledgeLocator) {
  const location = locator.location;
  if (location.kind !== "markdown" || !location.heading) return undefined;
  return findBlock(
    doc,
    (node) => node.type.name === "heading" && node.textContent.trim() === location.heading,
  );
}

function locateQuotedText(doc: ProseMirrorNode, locator: KnowledgeLocator) {
  const exact = locator.quote?.exact.trim();
  if (!exact) return undefined;
  const needle = exact.slice(0, Math.min(120, exact.length));
  return findBlock(doc, (node) => node.textContent.includes(needle));
}

function findBlock(doc: ProseMirrorNode, predicate: (node: ProseMirrorNode) => boolean) {
  let match: number | undefined;
  doc.descendants((node, position) => {
    if (match !== undefined) return false;
    if (predicate(node)) {
      match = position + 1;
      return false;
    }
  });
  return match;
}
