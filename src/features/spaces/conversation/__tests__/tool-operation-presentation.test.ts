import { describe, expect, it } from "vitest";

import type { ConversationToolOperation } from "../../contracts/generated/ConversationToolOperation";
import { toolOperationPresentation } from "../tool-operation-presentation";

function operation(overrides: Partial<ConversationToolOperation>): ConversationToolOperation {
  return {
    id: "operation-1",
    runId: "run-1",
    name: "run_command",
    source: { type: "command" },
    context: { command: "python3 analyze.py data.csv" },
    state: "prepared",
    approval: null,
    output: null,
    resolution: null,
    attempt: 1,
    ...overrides,
  };
}

describe("toolOperationPresentation", () => {
  it("asks about a command by the command line it would run and says it runs unconfined", () => {
    expect(toolOperationPresentation(operation({}))).toEqual({
      titleKey: "spaces.conversation.toolOperations.command.approval",
      descriptionKey: "spaces.conversation.toolOperations.command.unconfined",
      detail: "python3 analyze.py data.csv",
    });
  });

  it("asks to check a command's effects when its outcome is unknown", () => {
    const presentation = toolOperationPresentation(
      operation({ state: "outcome_unknown", approval: "allow" }),
    );
    expect(presentation.titleKey).toBe("spaces.conversation.toolOperations.outcomeUnknown.title");
    expect(presentation.descriptionKey).toBe("spaces.conversation.toolOperations.command.outcomeUnknown");
    expect(presentation.detail).toBe("python3 analyze.py data.csv");
  });

  it("shows any other operation as its recorded call", () => {
    const source = {
      type: "mcp_tool",
      connectionId: "c",
      toolName: "create_issue",
      schemaDigest: "d",
    } as const;
    const presentation = toolOperationPresentation(
      operation({ name: "mcp_c_create_issue", source, context: { title: "Bug" } }),
    );
    expect(presentation.titleKey).toBe("spaces.conversation.toolOperations.approval.title");
    expect(presentation.descriptionKey).toBeUndefined();
    expect(JSON.parse(presentation.detail)).toEqual({ source, context: { title: "Bug" }, output: null });
  });
});
