import { describe, expect, it } from "vitest";

import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import { conversationTaskRequest } from "../conversation/conversation-task-request";

describe("Agent conversation submission", () => {
  it.each<TaskTargetRef>([
    { type: "model", connectionId: "connection", modelId: "model" },
    { type: "agent", runtimeKind: "codex" },
  ])("submits the task and frozen extension selection for $type", (target) => {
    const extensions = {
      pluginIds: [],
      skillIds: ["skill"],
      connections: [
        {
          connectionId: "connection",
          toolNames: ["publish"],
          resourceUris: [],
          resourceTemplates: [],
          promptNames: [],
        },
      ],
    };
    const request = conversationTaskRequest({
      spaceId: "space",
      conversation: {
        id: "conversation",
        spaceId: "space",
        branchId: "branch",
        title: "Notes",
        updatedAt: 1,
      },
      prompt: "  Organize these notes  ",
      target,
      attachments: [],
      extensions,
    })!;
    expect(request).toEqual({
      spaceId: "space",
      conversationId: "conversation",
      branchId: "branch",
      prompt: "Organize these notes",
      target: target.type === "model" ? { ...target, temperature: 0.2 } : target,
      attachments: [],
      extensions,
    });
    extensions.connections[0]!.toolNames.push("another-tool");
    expect(request.extensions.connections[0]!.toolNames).toEqual(["publish"]);
  });
});
