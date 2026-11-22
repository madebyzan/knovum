import { describe, expect, it } from "vitest";

import { createSpacePlacement } from "../canvas/navigation-requests";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { createSpaceState } from "../session/session-state";
import { createRightSidebarActions } from "../workspace/right-sidebar-actions";
import { openTestSpaceTab } from "./support/session-context";

function setup() {
  const state = createSpaceState();
  openTestSpaceTab(state, "inbox");
  state.nodes.value = [
    createSpaceNode(createSpaceNotePayload("Research"), { x: 0, y: 0 }, "note-1"),
    createSpaceNode(
      { kind: "video", title: "Interview", fileName: "a.mp4", assetPath: "/a.mp4" },
      { x: 0, y: 0 },
      "video-1",
    ),
    createSpaceNode(
      { kind: "object", title: "Claim", objectTypeId: "claim", status: "open", properties: {} },
      { x: 0, y: 0 },
      "object-1",
    ),
  ];
  const placement = createSpacePlacement({
    insertionPoint: state.insertionPoint,
    revealRequest: state.revealRequest,
    viewportRequest: state.viewportRequest,
    zoomRequest: state.zoomRequest,
    selectionClearNonce: state.selectionClearNonce,
    sceneCue: state.sceneCue,
  });
  return {
    state,
    actions: createRightSidebarActions(state, placement, async () => {}),
  };
}

describe("Space context dock and reader actions", () => {
  it("opens Ask with an explicit contextual scope request", () => {
    const { state, actions } = setup();
    actions.requestAsk("selection");
    expect(state.askScopeRequest.value).toMatchObject({ scope: "selection", restart: false });
  });

  it("routes media citations directly to the unified Source Reader without triggering canvas playback", () => {
    const { state, actions } = setup();
    actions.openSource(
      {
        nodeId: "video-1",
        nodeTitle: "Interview",
        locator: { sourceRevision: "1", location: { kind: "media", startMs: 1200, endMs: 2400 } },
        text: "Key quote",
      },
      { reveal: false },
    );
    expect(state.readingTarget.value).toMatchObject({
      nodeId: "video-1",
      pane: "read",
      quote: "Key quote",
      locator: { sourceRevision: "1", location: { kind: "media", startMs: 1200, endMs: 2400 } },
    });
    expect(state.mediaSeekRequest.value).toBeUndefined();
  });

  it("opens non-readable objects in info pane and supports history back/forward", () => {
    const { state, actions } = setup();
    actions.openSourceReader({ nodeId: "note-1", pane: "read" });
    actions.openSourceReader({ nodeId: "object-1" });
    expect(state.readingTarget.value).toMatchObject({ nodeId: "object-1", pane: "info" });

    actions.stepReadingHistory(-1);
    expect(state.readingTarget.value).toMatchObject({ nodeId: "note-1", pane: "read" });

    actions.stepReadingHistory(1);
    expect(state.readingTarget.value).toMatchObject({ nodeId: "object-1", pane: "info" });
  });

  it("reveals cited nodes without changing the user's canvas scale", () => {
    const { state, actions } = setup();
    actions.openSource({
      nodeId: "note-1",
      nodeTitle: "Research",
      locator: { sourceRevision: "1", location: { kind: "note" } },
    });
    expect(state.revealRequest.value).toMatchObject({
      ids: ["note-1"],
      mode: "ensure-visible",
    });
  });
});
