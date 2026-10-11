import { describe, expect, it, vi } from "vitest";

import { applyDerivationOutput } from "../knowledge-derivations/apply-output";
import type { SpaceNodePayload } from "../model/nodes/node-payload";

const current = vi.hoisted(() => ({ store: undefined as unknown }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => current.store }));

const data = { kind: "markdown", title: "Result", text: "Done" } as const;

describe("derivation output application", () => {
  it("applies returned relations and reveals the node in the active space", () => {
    const store = harness("space-a");
    applyDerivationOutput({
      spaceId: "space-a",
      nodeId: "result",
      data,
      relations: [relation("space-a")],
    });

    expect(store.relations).toHaveLength(1);
    expect(store.applyNodeContentForSpace).toHaveBeenCalledWith("space-a", "result", data);
    expect(store.requestReveal).toHaveBeenCalledWith(["result"], "ensure-visible");
  });

  it("does not mutate or reveal when the completed job belongs to another space", () => {
    const store = harness("space-b");
    expect(
      applyDerivationOutput({
        spaceId: "space-a",
        nodeId: "result",
        data,
        relations: [relation("space-a")],
      }),
    ).toBe(false);
    expect(store.relations).toEqual([]);
    expect(store.applyNodeContentForSpace).not.toHaveBeenCalled();
    expect(store.requestReveal).not.toHaveBeenCalled();
  });
});

function harness(activeSpaceId: string) {
  return (current.store = {
    activeSpaceId,
    relations: [] as ReturnType<typeof relation>[],
    applyNodeContentForSpace: vi.fn<(spaceId: string, nodeId: string, value: SpaceNodePayload) => void>(),
    requestReveal: vi.fn<(nodeIds: readonly string[], mode?: "ensure-visible" | "focus") => void>(),
  });
}

function relation(spaceId: string) {
  return {
    id: "relation",
    spaceId,
    sourceId: "source",
    targetId: "result",
    kind: "derives-from" as const,
    rationale: "",
    status: "active" as const,
    actorKind: "system" as const,
    createdAt: 1,
    updatedAt: 1,
  };
}
