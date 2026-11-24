import { isSpaceSemanticNode } from "../knowledge/model/semantic-node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodeKind } from "../model/nodes/node-payload";
import type { ConversationAttachmentCandidatePresentation } from "./conversation-attachment-presentation";

export type SpaceSourceCategory = "notes" | "documents" | "webpages" | "media" | "objects";

export interface SpaceQuickContextItem {
  readonly nodeId: string;
  readonly title: string;
  readonly nodeKind: SpaceNodeKind;
  readonly previewSource: string;
  readonly selected: boolean;
  readonly attached: boolean;
}

export const SPACE_SOURCE_CATEGORY_LABEL_KEY: Readonly<Record<SpaceSourceCategory, string>> = {
  notes: "spaces.conversation.empty.sources.notes",
  documents: "spaces.conversation.empty.sources.documents",
  webpages: "spaces.conversation.empty.sources.webpages",
  media: "spaces.conversation.empty.sources.media",
  objects: "spaces.conversation.empty.sources.objects",
};

const SOURCE_CATEGORY_BY_NODE_KIND: Readonly<Partial<Record<SpaceNodeKind, SpaceSourceCategory>>> = {
  note: "notes",
  markdown: "notes",
  file: "documents",
  code: "documents",
  website: "webpages",
  image: "media",
  video: "media",
  audio: "media",
  object: "objects",
};

export function summarizeSpaceSources(nodes: readonly SpaceCanvasNode[]) {
  const counts = new Map<SpaceSourceCategory, number>();
  let totalCount = 0;
  for (const node of nodes) {
    if (!isSpaceSemanticNode(node)) continue;
    totalCount += 1;
    const category = SOURCE_CATEGORY_BY_NODE_KIND[node.data.kind];
    if (category) counts.set(category, (counts.get(category) ?? 0) + 1);
  }
  const categories = [...counts.entries()]
    .map(([category, count]) => ({ category, count }))
    .toSorted((left, right) => right.count - left.count)
    .slice(0, 3);
  return { totalCount, categories };
}

function contextPriority(candidate: ConversationAttachmentCandidatePresentation) {
  if (candidate.selected) return 0;
  if (candidate.attached) return 1;
  return 2;
}

export function selectQuickContextItems(
  candidates: readonly ConversationAttachmentCandidatePresentation[],
  nodes: readonly SpaceCanvasNode[],
  limit = 5,
): readonly SpaceQuickContextItem[] {
  const kindByNodeId = new Map(nodes.map((node) => [node.id, node.data.kind]));
  return candidates
    .toSorted((left, right) => contextPriority(left) - contextPriority(right))
    .slice(0, limit)
    .flatMap((candidate) => {
      const nodeKind = kindByNodeId.get(candidate.nodeId);
      if (!nodeKind) return [];
      return [{ ...candidate, nodeKind }];
    });
}

export function formatRecentConversationTimestamp(
  updatedAt: number,
  locale: string,
  yesterdayLabel: string,
  now = new Date(),
): string {
  const date = new Date(updatedAt);
  if (date.toDateString() === now.toDateString())
    return new Intl.DateTimeFormat(locale, { hour: "2-digit", minute: "2-digit" }).format(date);
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  return date.toDateString() === yesterday.toDateString()
    ? yesterdayLabel
    : new Intl.DateTimeFormat(locale, { month: "short", day: "numeric" }).format(date);
}
