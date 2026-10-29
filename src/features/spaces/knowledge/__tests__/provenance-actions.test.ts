import { beforeEach, expect, it, vi } from "vitest";

import { queryClient } from "../../../../foundation/query/query-client";
import type { SpaceProvenance } from "../model/provenance";
import { attachProvenanceFromPeek, reviewProvenance } from "../provenance-actions";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "../queries/query-options";

const invoke = vi.hoisted(() => vi.fn<(command: string, arguments_?: unknown) => Promise<void>>());
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));

beforeEach(() => {
  invoke.mockReset();
  invoke.mockResolvedValue();
});

it("persists canonical provenance from a peek and refreshes its target projection", async () => {
  const invalidate = vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue();
  await attachProvenanceFromPeek(
    { spaceId: "space", targetKind: "node", targetId: "evidence" },
    {
      nodeId: "source",
      nodeTitle: "Source",
      text: "fallback",
      locator: {
        sourceRevision: "revision-1",
        location: { kind: "note", blockId: "block" },
        quote: { exact: "quoted evidence" },
      },
    },
  );

  expect(invoke).toHaveBeenCalledWith("plugin:spaces|create_provenance", {
    input: {
      id: expect.any(String),
      spaceId: "space",
      targetKind: "node",
      targetId: "evidence",
      sourceNodeId: "source",
      locator: {
        sourceRevision: "revision-1",
        location: { kind: "note", blockId: "block" },
        quote: { exact: "quoted evidence" },
      },
      sourceRevision: "revision-1",
      quote: "quoted evidence",
    },
  });
  expect(invalidate).toHaveBeenCalledWith({
    queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.provenance("space", "node", "evidence"),
  });
  expect(invalidate).toHaveBeenCalledWith({ queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.health("space") });
});

it("uses the peek text when its locator has no exact quote", async () => {
  vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue();
  await attachProvenanceFromPeek(
    { spaceId: "space", targetKind: "relation", targetId: "relation" },
    {
      nodeId: "source",
      nodeTitle: "Source",
      text: "visible passage",
      locator: { sourceRevision: "revision-2", location: { kind: "node" } },
    },
  );

  expect(invoke.mock.calls[0]?.[1]).toMatchObject({
    input: { sourceRevision: "revision-2", quote: "visible passage" },
  });
});

it("re-takes a reviewed record at the source's new revision, keeping its words", async () => {
  vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue();
  const record: SpaceProvenance = {
    id: "old",
    spaceId: "space",
    targetKind: "node",
    targetId: "note",
    sourceNodeId: "source",
    locator: { sourceRevision: "r1", location: { kind: "note", blockId: "b" } },
    sourceRevision: "r1",
    quote: "what was kept",
    status: "active",
    actorKind: "human",
    createdAt: 1,
    updatedAt: 1,
  };
  await reviewProvenance(record, "r2");

  expect(invoke.mock.calls[0]?.[1]).toMatchObject({
    input: {
      targetId: "note",
      sourceNodeId: "source",
      sourceRevision: "r2",
      locator: { sourceRevision: "r2", location: { kind: "node" } },
      quote: "what was kept",
    },
  });
});

it("re-takes a reviewed record at the source's new revision, keeping the words and the whole source", async () => {
  vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue();
  await reviewProvenance(
    {
      id: "old",
      spaceId: "space",
      targetKind: "node",
      targetId: "note",
      sourceNodeId: "source",
      locator: { sourceRevision: "r1", location: { kind: "note", blockId: "block" } },
      sourceRevision: "r1",
      quote: "kept words",
      status: "active",
      actorKind: "human",
      createdAt: 1,
      updatedAt: 1,
    },
    "r2",
  );

  expect(invoke.mock.calls[0]?.[1]).toMatchObject({
    input: {
      targetId: "note",
      sourceNodeId: "source",
      locator: { sourceRevision: "r2", location: { kind: "node" } },
      sourceRevision: "r2",
      quote: "kept words",
    },
  });
});
