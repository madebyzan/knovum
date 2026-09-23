import type { WorkflowNode, WorkflowNodeKind } from "./workflow-client";

export const WORKFLOW_NODE_KINDS = [
  "action",
  "condition",
  "parallel",
  "join",
  "for_each",
  "iterate",
  "subworkflow",
  "human_input",
  "review",
  "delay",
  "data_transform",
  "external_tool",
  "delegate_task",
  "publish",
] as const satisfies readonly WorkflowNodeKind[];

type NodeFactory = (id: string) => WorkflowNode;
const nodeFactories = {
  input: (id) => ({ kind: "input", id, outputType: "record" }),
  output: (id) => ({ kind: "output", id, inputType: "record" }),
  action: (id) => ({
    kind: "action",
    id,
    actionId: "create-note",
    actionVersion: 1,
    parameters: { title: "New note", content: "Created by Workflow" },
  }),
  condition: (id) => ({ kind: "condition", id, expression: "input != null" }),
  parallel: (id) => ({ kind: "parallel", id }),
  join: (id) => ({ kind: "join", id, policy: "active_branches" }),
  for_each: (id) => ({ kind: "for_each", id, maxItems: 1000, parallelism: 4, errorPolicy: "stop" }),
  iterate: (id) => ({ kind: "iterate", id, maxIterations: 3, stopExpression: "input.done == true" }),
  subworkflow: (id) => ({ kind: "subworkflow", id, workflowId: "", version: 1 }),
  human_input: (id) => ({ kind: "human_input", id, schema: { type: "object" } }),
  review: (id) => ({ kind: "review", id, purpose: "Review the generated result" }),
  delay: (id) => ({ kind: "delay", id, durationSeconds: 60 }),
  data_transform: (id) => ({ kind: "data_transform", id, expression: "input" }),
  external_tool: (id) => ({ kind: "external_tool", id, bindingId: "", schemaFingerprint: "" }),
  delegate_task: (id) => ({ kind: "delegate_task", id, runtimeId: "" }),
  publish: (id) => ({ kind: "publish", id, mode: "create_resource", destination: null }),
} satisfies Record<WorkflowNodeKind, NodeFactory>;

export function createWorkflowNode(kind: WorkflowNodeKind, id: string): WorkflowNode {
  return nodeFactories[kind](id);
}
