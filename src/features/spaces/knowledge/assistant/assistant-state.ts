import type { GroundedScope } from "../model/grounded-scope";
import type { GroundedThread } from "../model/grounded-thread";

export function suggestedKnowledgeScope(
  selectedIds: readonly string[],
  sectionIds: readonly string[],
): GroundedScope {
  if (selectedIds.length) return "selection";
  if (sectionIds.length) return "section";
  return "space";
}

export function knowledgeScopeNodeIds(
  scope: GroundedScope,
  selectedIds: readonly string[],
  sectionIds: readonly string[],
) {
  if (scope === "selection") return [...selectedIds];
  if (scope === "section") return [...sectionIds];
  return [];
}

export function activeThreadNodeIds(thread: GroundedThread | undefined, defaultNodeIds: readonly string[]) {
  if (!thread) return [...defaultNodeIds];
  return thread.scope === "space" || thread.scope === "all-spaces" ? [] : [...thread.scopeNodeIds];
}
