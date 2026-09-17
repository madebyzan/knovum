import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn<
    (
      command: string,
      args: { request: { workflowId: string; steps: { id: string; status: string }[] } },
    ) => Promise<string | null>
  >(
    async (
      _command: string,
      { request }: { request: { workflowId: string; steps: { id: string; status: string }[] } },
    ) => {
      if (request.workflowId === "cycle") throw new Error("Creation Workflow dependency cycle");
      return request.steps.find(({ status }) => status === "pending")?.id ?? null;
    },
  ),
}));

import { executeCreationWorkflow } from "../knowledge-derivations/orchestration/workflow-executor";

describe("creation workflow executor", () => {
  it("runs dependent steps with explicit outputs", async () => {
    const order: string[] = [];
    const snapshot = await executeCreationWorkflow({
      runId: "run",
      spaceId: "space",
      workflow: {
        id: "test-workflow",
        version: 1,
        steps: [
          { id: "source", action: "create-source", inputs: {} },
          { id: "output", action: "create-output", inputs: { source: "source" } },
        ],
      },
      adapters: {
        "create-source": async ({ step }) => {
          order.push(step.id);
          return { output: step.id, outputNodeIds: ["source-node"] };
        },
        "create-output": async ({ step, inputs }) => {
          order.push(step.id);
          expect(inputs.source?.outputNodeIds).toEqual(["source-node"]);
          return { output: step.id, outputNodeIds: ["output-node"] };
        },
      },
    });

    expect(order).toEqual(["source", "output"]);
    expect(snapshot.steps.map(({ status }) => status)).toEqual(["succeeded", "succeeded"]);
  });

  it("rejects cycles before running any step", async () => {
    const runStep = vi.fn<() => Promise<{ output: undefined; outputNodeIds: string[] }>>(async () => ({
      output: undefined,
      outputNodeIds: [],
    }));
    await expect(
      executeCreationWorkflow({
        runId: "cycle-run",
        spaceId: "space",
        workflow: {
          id: "cycle",
          version: 1,
          steps: [
            { id: "a", action: "a", inputs: { source: "b" } },
            { id: "b", action: "b", inputs: { source: "a" } },
          ],
        },
        adapters: {
          a: runStep,
          b: runStep,
        },
      }),
    ).rejects.toThrow("dependency cycle");
    expect(runStep).not.toHaveBeenCalled();
  });
});
