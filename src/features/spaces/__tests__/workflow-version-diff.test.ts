import { describe, expect, it } from "vitest";

import type { WorkflowDefinition } from "../workflows/workflow-client";
import { diffWorkflowVersions } from "../workflows/workflow-version-diff";

function workflow(): WorkflowDefinition {
  return {
    schemaVersion: 1,
    title: "Research",
    nodes: [
      { kind: "input", id: "input", outputType: "value" },
      { kind: "output", id: "output", inputType: "value" },
    ],
    edges: [
      {
        id: "edge",
        sourceNodeId: "input",
        sourcePort: "output",
        targetNodeId: "output",
        targetPort: "input",
        branch: null,
      },
    ],
    layout: {},
    failurePolicy: "stop_run",
    limits: {
      maxExpandedSteps: 100,
      maxParallelism: 4,
      maxDurationSeconds: 60,
      maxTotalTokens: 1_000,
      maxStepAttempts: 10,
      maxInputBytes: 1_024,
      maxOutputBytes: 1_024,
    },
  };
}

describe("workflow version diff", () => {
  it("reports semantic graph and policy changes without layout noise", () => {
    const previous = workflow();
    const current = structuredClone(previous);
    current.layout.input = { x: 20, y: 40 };
    current.nodes.splice(1, 0, {
      kind: "condition",
      id: "gate",
      expression: "input.ready == true",
    });
    current.edges = [];
    current.failurePolicy = "collect_errors";

    expect(diffWorkflowVersions(previous, current)).toEqual({
      addedNodes: ["gate · condition"],
      removedNodes: [],
      changedNodes: [],
      addedEdges: [],
      removedEdges: ["input.output → output.input"],
      policyChanges: ["failurePolicy: stop_run → collect_errors"],
    });
  });
});
