import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export interface ConversationAttachmentPresentation {
  key: string;
  kind: "node" | "file";
  title: string;
  isImage: boolean;
  previewSource: string;
}

export interface ConversationAttachmentCandidatePresentation {
  nodeId: string;
  title: string;
  previewSource: string;
  selected: boolean;
  attached: boolean;
}

/**
 * The file type a document attachment is shown with, read from its name ("deck.pptx" → "PPTX");
 * undefined when the name carries none, as a canvas node's title usually does.
 */
export function attachmentFileType(title: string): string | undefined {
  return /\.((?=[a-z0-9]{1,5}$)[a-z0-9]*[a-z][a-z0-9]*)$/i.exec(title.trim())?.[1]?.toUpperCase();
}

function buildNodeRevisionMap(nodes: readonly SpaceCanvasNode[]): Map<string, string> {
  const revisionById = new Map<string, string>();
  for (const node of nodes) {
    const revision = node.data.knowledgeRevision ?? node.data.sourceRevision;
    if (revision) revisionById.set(node.id, revision);
  }
  return revisionById;
}

export function attachmentKey(item: ConversationAttachment): string {
  return item.kind === "node" ? `node:${item.nodeId}` : `file:${item.path}`;
}

/**
 * Reconciles attached node source revisions against authoritative Space nodes.
 * Returns a new attachment array only when at least one attachment changed or was dropped.
 */
export function reconcileNodeAttachmentRevisions(
  attachments: readonly ConversationAttachment[],
  nodes: readonly SpaceCanvasNode[],
  options: { readonly dropMissing?: boolean } = {},
): ConversationAttachment[] | undefined {
  if (!attachments.some((item) => item.kind === "node")) return undefined;
  const revisionById = buildNodeRevisionMap(nodes);
  let changed = false;
  const next: ConversationAttachment[] = [];
  for (const item of attachments) {
    if (item.kind !== "node") {
      next.push(item);
      continue;
    }
    const latest = revisionById.get(item.nodeId);
    if (!latest) {
      if (options.dropMissing) changed = true;
      else next.push(item);
    } else if (latest !== item.sourceRevision) {
      changed = true;
      next.push({ kind: "node", nodeId: item.nodeId, sourceRevision: latest, role: item.role });
    } else next.push(item);
  }
  return changed ? next : undefined;
}
