import { beforeEach, expect, it, vi } from "vitest";

import { queryClient } from "../../../../foundation/query/query-client";
import { attachProvenanceFromPeek } from "../provenance-actions";
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
