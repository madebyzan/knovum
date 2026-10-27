import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invokeCommand: vi.fn<(command: string, args: unknown) => Promise<unknown>>(),
  openSpaceNode: vi.fn<(node: unknown) => Promise<void>>(),
  requestReveal: vi.fn<(nodeIds: string[], mode: string) => void>(),
  nodes: [] as { id: string; data: { kind: string } }[],
}));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invokeCommand }));
vi.mock("../../canvas/selection-actions", () => ({
  isSpaceNodeOpenable: (node?: { data: { kind: string } }) => node?.data.kind === "markdown",
  openSpaceNode: mocks.openSpaceNode,
}));
vi.mock("../../session/space-store", () => ({
  useSpaceStore: () => ({ nodes: mocks.nodes, requestReveal: mocks.requestReveal }),
}));

import { followConversationLink } from "../conversation-links";

beforeEach(() => {
  mocks.invokeCommand.mockReset();
  mocks.openSpaceNode.mockReset().mockResolvedValue();
  mocks.requestReveal.mockReset();
  mocks.nodes = [
    { id: "summary", data: { kind: "markdown" } },
    { id: "folder", data: { kind: "folder" } },
  ];
});

describe("followConversationLink", () => {
  it("asks Rust to resolve the link from the message that holds it", async () => {
    mocks.invokeCommand.mockResolvedValue({ kind: "opened" });
    await followConversationLink("answer", "/Users/zand/deck.pptx");
    expect(mocks.invokeCommand).toHaveBeenCalledExactlyOnceWith("plugin:spaces|follow_conversation_link", {
      messageId: "answer",
      href: "/Users/zand/deck.pptx",
    });
    expect(mocks.openSpaceNode).not.toHaveBeenCalled();
  });

  it("opens a Knovum document the way the canvas opens it", async () => {
    mocks.invokeCommand.mockResolvedValue({ kind: "node", nodeId: "summary" });
    await followConversationLink("answer", "PPT%E6%91%98%E8%A6%81.md");
    expect(mocks.openSpaceNode).toHaveBeenCalledExactlyOnceWith(mocks.nodes[0]);
  });

  it("shows a node that cannot be opened where it is on the canvas", async () => {
    mocks.invokeCommand.mockResolvedValue({ kind: "node", nodeId: "folder" });
    await followConversationLink("answer", "out");
    expect(mocks.requestReveal).toHaveBeenCalledExactlyOnceWith(["folder"], "focus");
  });
});
