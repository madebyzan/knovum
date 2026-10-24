import type { KnowledgeIndexState } from "../../contracts/generated/KnowledgeIndexState";

/**
 * What the index panel says, and what its one button would do.
 *
 * The reading is derived, never stored: `state` already describes the decision the next pass will
 * make, so the panel reports that decision instead of inventing a second one. An index with no
 * gap offers nothing to press — a Space that is already searchable has nothing to repair, and a
 * button that re-encodes it anyway is a second entry point that can only disagree with this one.
 */
type KnowledgeIndexCondition =
  /** The Space carries no text worth indexing. */
  | "empty"
  /** A pass holds this Space right now. */
  | "indexing"
  /** Semantic search cannot answer from this Space: the next pass encodes all of it. */
  | "unbuilt"
  /** Searchable, with content the index has not caught up with. */
  | "incomplete"
  /** Every chunk of the Space is in the index that answers searches. */
  | "current";

export interface KnowledgeIndexReading {
  condition: KnowledgeIndexCondition;
  /** Share of the Space a search can reach, 0–100. */
  coverage: number;
  indexed: number;
  pending: number;
  total: number;
  /** The pass the button would start; absent when there is nothing to ask for. */
  action: "build" | "fill" | undefined;
  indexedAt: number | undefined;
}

const ACTION_BY_CONDITION = {
  empty: undefined,
  indexing: undefined,
  unbuilt: "build",
  incomplete: "fill",
  current: undefined,
} as const satisfies Record<KnowledgeIndexCondition, "build" | "fill" | undefined>;

export function readKnowledgeIndex(
  state: KnowledgeIndexState | undefined,
  passRunning: boolean,
): KnowledgeIndexReading {
  const total = state?.total ?? 0;
  const pending = Math.min(state?.pending ?? 0, total);
  const condition = conditionOf(state, pending, passRunning);
  return {
    condition,
    coverage: total > 0 ? Math.round(((total - pending) / total) * 100) : 100,
    indexed: total - pending,
    pending,
    total,
    action: ACTION_BY_CONDITION[condition],
    indexedAt: state?.indexedAt ?? undefined,
  };
}

function conditionOf(
  state: KnowledgeIndexState | undefined,
  pending: number,
  passRunning: boolean,
): KnowledgeIndexCondition {
  // A pass that is already running holds the Space, whatever the snapshot said about its gaps.
  if (passRunning || state?.indexing) return "indexing";
  if (!state || state.total === 0) return "empty";
  if (!state.searchable) return "unbuilt";
  return pending > 0 ? "incomplete" : "current";
}
