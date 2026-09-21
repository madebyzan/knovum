import { describe, expect, it } from "vitest";

import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { parseSpacePackageManifest } from "../package/manifest-parser";

function manifest() {
  const data = prepareSpaceKnowledgePayload({
    kind: "markdown",
    title: "N",
    text: "body",
  } as SpaceNodePayload);
  const revision = data.knowledgeRevision as string;
  return {
    format: "knovum-space",
    version: 1,
    exportedAt: 10,
    structureNodes: [{ id: "n", parentId: null, order: 0, locked: false, kind: "content", nodeId: "n" }],
    layouts: [
      {
        structureNodeId: "n",
        kind: "content",
        transform: { x: 0, y: 0, rotation: 0, scale: 1 },
        size: { width: 320, height: 180 },
        sizeAuthority: "frame",
      },
    ],
    relationStyles: [],
    space: {
      id: "s",
      kind: "space",
      title: "S",
      archivedAt: null,
      lastOpenedAt: 1,
      createdAt: 1,
      updatedAt: 1,
    },
    nodes: [{ id: "n", kind: "markdown", data, createdAt: 1, updatedAt: 1 }],
    scenes: [],
    relations: [],
    derivations: [],
    creationWorkflowRuns: [],
    chunks: [],
    objectTypes: [],
    provenance: [],
    knowledgeEvents: [],
    aiThreads: [
      { id: "t", spaceId: "s", scope: "space", scopeNodeIds: [], title: "T", createdAt: 1, updatedAt: 1 },
    ],
    aiMessages: [
      {
        id: "m",
        threadId: "t",
        role: "assistant",
        content: "answer",
        createdAt: 1,
        citations: [
          {
            index: 1,
            state: "current",
            chunkId: "node:n",
            nodeId: "n",
            nodeTitle: "N",
            sourceRevision: revision,
            locator: { sourceRevision: revision, location: { kind: "node" } },
            quote: "N",
          },
          {
            index: 2,
            state: "detached",
            sourceSpaceTitle: "Archive",
            nodeTitle: "Old",
            sourceRevision: "node1:old",
            locator: { sourceRevision: "node1:old", location: { kind: "node" } },
            quote: "historical",
          },
        ],
      },
    ],
  };
}

describe("Space package manifest parser", () => {
  it("accepts regenerated canonical citations and detached historical snapshots", () => {
    const parsed = parseSpacePackageManifest(JSON.stringify(manifest()));
    expect(parsed.version).toBe(1);
    expect(parsed.aiMessages[0]?.citations.map((citation) => citation.state)).toEqual([
      "current",
      "detached",
    ]);
  });

  it("accepts All-Spaces threads with detached external citation snapshots", () => {
    const value = manifest();
    value.aiThreads[0]!.scope = "all-spaces";
    expect(() => parseSpacePackageManifest(JSON.stringify(value))).not.toThrow();
  });

  it("rejects a canonical citation whose snapshot revision is stale", () => {
    const value = manifest();
    value.aiMessages[0]!.citations[0]!.sourceRevision = "node1:stale";
    value.aiMessages[0]!.citations[0]!.locator.sourceRevision = "node1:stale";
    expect(() => parseSpacePackageManifest(JSON.stringify(value))).toThrow(/canonical citation revision/u);
  });

  it("rejects derivations that reference unavailable nodes", () => {
    const value = manifest();
    (value.derivations as unknown[]).push({
      id: "d",
      spaceId: "s",
      outputNodeId: "missing",
      inputs: [{ nodeId: "n", role: "primary", sourceRevision: "node1:source" }],
      actionKind: "image-background-removal",
      actionVersion: 1,
      engine: "u2netp-tract:1",
      status: "completed",
      error: "",
      createdAt: 1,
      updatedAt: 2,
      completedAt: 2,
    });
    expect(() => parseSpacePackageManifest(JSON.stringify(value))).toThrow(/unavailable node/u);
  });

  it("rejects layout data that copies domain content into the structure contract", () => {
    const value = manifest();
    Object.assign(value.layouts[0]!, { title: "duplicated content" });

    expect(() => parseSpacePackageManifest(JSON.stringify(value))).toThrow(/unrecognized_keys/u);
  });
});
