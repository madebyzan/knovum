import type { KnowledgeActorKind } from "../../contracts/generated/KnowledgeActorKind";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { KnowledgeLocator } from "./schema/chunk";

type SpaceProvenanceStatus = "active" | "superseded" | "invalidated";
export type SpaceProvenanceTargetKind = "node" | "relation";

export interface SpaceProvenance {
  id: string;
  spaceId: string;
  targetKind: SpaceProvenanceTargetKind;
  targetId: string;
  sourceNodeId: string;
  locator: KnowledgeLocator;
  sourceRevision: string;
  quote: string;
  status: SpaceProvenanceStatus;
  actorKind: KnowledgeActorKind;
  actorId?: string;
  createdAt: number;
  updatedAt: number;
}

/** Where something was taken from: the source item, the place in it and the words. */
export interface ProvenanceSource {
  nodeId: string;
  locator: KnowledgeLocator;
  quote: string;
}

type SourceRevisions = Pick<SpaceNodePayload, "sourceRevision" | "knowledgeRevision">;

/**
 * Whether a source still has the revision a record was taken at. A cited passage carries the revision
 * of what was indexed — the bytes of a file or recording, the content of a note or page — so the
 * record is current while the source has that revision on either axis. A source without revisions
 * has nothing newer to report.
 */
export function sourceHasRevision(source: SourceRevisions, revision: string) {
  const current = [source.sourceRevision, source.knowledgeRevision].filter(Boolean);
  return current.length === 0 || current.includes(revision);
}

/** The records whose source, still in `sources`, has changed since the record was taken. */
export function changedSources<T extends { sourceNodeId?: string; sourceRevision?: string }>(
  records: readonly T[],
  sources: ReadonlyMap<string, SourceRevisions>,
) {
  return records.filter(({ sourceNodeId, sourceRevision }) => {
    const source = sourceNodeId === undefined ? undefined : sources.get(sourceNodeId);
    return source !== undefined && sourceRevision !== undefined && !sourceHasRevision(source, sourceRevision);
  });
}

/** The revision a record is re-taken at: the source's bytes where it has them, else its content. */
export function currentSourceRevision(source: SourceRevisions) {
  return source.sourceRevision ?? source.knowledgeRevision;
}

type SpaceKnowledgeEventKind =
  | "created"
  | "updated"
  | "drafted"
  | "accepted"
  | "contested"
  | "superseded"
  | "invalidated"
  | "opened"
  | "resolved"
  | "restored"
  | "linked"
  | "unlinked"
  | "applied"
  | "rejected"
  | "moved";

export interface SpaceKnowledgeEvent {
  id: string;
  spaceId: string;
  entityKind: "node" | "relation" | "provenance" | "type";
  entityId: string;
  eventKind: SpaceKnowledgeEventKind;
  actorKind: KnowledgeActorKind;
  actorId?: string;
  payload: unknown;
  createdAt: number;
}
