import { isString, keyBy } from "es-toolkit";

import type { GroundedCitation } from "../knowledge/model/grounded-citation";
import { knowledgeRevision } from "../knowledge/model/revision";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import type { SpacePackageManifest } from "./manifest-schema";

type NodeRevisions = Map<string, { knowledge?: string; source?: string }>;
type CurrentCitation = Extract<GroundedCitation, { state: "current" }>;
type CitationIndexes = {
  threads: Record<string, SpacePackageManifest["aiThreads"][number]>;
  nodeRevisions: Map<string, string | undefined>;
  chunks: Record<string, SpacePackageManifest["chunks"][number]>;
};

export function validatePackageKnowledgeSemantics(manifest: SpacePackageManifest) {
  validateSpacePackageChunkSemantics(manifest);
  validateSpacePackageCitationSemantics(manifest);
}

function validateSpacePackageChunkSemantics(manifest: SpacePackageManifest) {
  const revisions = buildNodeRevisions(manifest);
  for (const chunk of manifest.chunks) validateChunkRevision(chunk, revisions);
}

function buildNodeRevisions(manifest: SpacePackageManifest): NodeRevisions {
  const revisions: NodeRevisions = new Map();
  for (const node of manifest.nodes) {
    const data = validateSpaceNodePayload(node.data);
    if (data.kind !== node.kind) throw new Error("Space package node kind does not match node data");
    const authoritative = knowledgeRevision(data);
    if (authoritative !== data.knowledgeRevision) {
      throw new Error("Space package contains a non-authoritative Knowledge revision");
    }
    revisions.set(node.id, {
      ...(authoritative ? { knowledge: authoritative } : {}),
      ...(isString(data.sourceRevision) ? { source: data.sourceRevision } : {}),
    });
  }
  return revisions;
}

function validateChunkRevision(chunk: SpacePackageManifest["chunks"][number], revisions: NodeRevisions) {
  const revision = revisions.get(chunk.nodeId);
  const expected = chunk.origin === "derived" ? revision?.knowledge : revision?.source;
  if (!expected || chunk.sourceRevision !== expected) {
    throw new Error("Space package contains a stale Knowledge passage revision");
  }
  if (chunk.locator.sourceRevision !== chunk.sourceRevision) {
    throw new Error("Space package Knowledge passage revision does not match its locator");
  }
}

function validateSpacePackageCitationSemantics(manifest: SpacePackageManifest) {
  const indexes = buildCitationIndexes(manifest);
  for (const message of manifest.aiMessages) validateMessageCitations(message, indexes);
}

function buildCitationIndexes(manifest: SpacePackageManifest): CitationIndexes {
  return {
    threads: keyBy(manifest.aiThreads, (thread) => thread.id),
    nodeRevisions: new Map(
      manifest.nodes.map((node) => {
        const data = validateSpaceNodePayload(node.data);
        return [node.id, data.knowledgeRevision] as const;
      }),
    ),
    chunks: keyBy(manifest.chunks, (chunk) => chunk.id),
  };
}

function validateMessageCitations(
  message: SpacePackageManifest["aiMessages"][number],
  indexes: CitationIndexes,
) {
  const thread = indexes.threads[message.threadId];
  if (!thread) throw new Error("Space package Grounded AI message references an unavailable thread");
  const scopedNodes =
    thread.scope === "space" || thread.scope === "all-spaces" ? undefined : new Set(thread.scopeNodeIds);
  for (const citation of message.citations) validateCitation(citation, scopedNodes, indexes);
}

function validateCitation(
  citation: SpacePackageManifest["aiMessages"][number]["citations"][number],
  scopedNodes: Set<string> | undefined,
  indexes: CitationIndexes,
) {
  validateCitationLocatorRevision(citation);
  if (citation.state === "detached") return;
  validateCurrentCitationNode(citation.nodeId, scopedNodes, indexes.nodeRevisions);
  if (citation.chunkId === `node:${citation.nodeId}`)
    return validateCanonicalCitation(citation, indexes.nodeRevisions);
  validatePassageCitation(citation, indexes.chunks);
}

function validateCitationLocatorRevision(
  citation: SpacePackageManifest["aiMessages"][number]["citations"][number],
) {
  if (citation.locator.sourceRevision !== citation.sourceRevision) {
    throw new Error("Space package citation source revision does not match its locator");
  }
}

function validateCurrentCitationNode(
  nodeId: string,
  scopedNodes: Set<string> | undefined,
  nodeRevisions: Map<string, string | undefined>,
) {
  if (!nodeRevisions.has(nodeId))
    throw new Error("Space package current citation references a missing packaged node");
  if (scopedNodes && !scopedNodes.has(nodeId)) {
    throw new Error("Space package current citation falls outside its persisted Grounded AI scope");
  }
}

function validateCanonicalCitation(
  citation: CurrentCitation,
  nodeRevisions: Map<string, string | undefined>,
) {
  if (nodeRevisions.get(citation.nodeId) !== citation.sourceRevision) {
    throw new Error("Space package canonical citation revision does not match its packaged node");
  }
}

function validatePassageCitation(
  citation: CurrentCitation,
  chunks: Record<string, SpacePackageManifest["chunks"][number]>,
) {
  const chunk = chunks[citation.chunkId];
  if (!chunk || chunk.nodeId !== citation.nodeId || chunk.sourceRevision !== citation.sourceRevision) {
    throw new Error("Space package current citation does not reference a current packaged Knowledge passage");
  }
}
