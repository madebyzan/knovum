import { describe, expect, it, vi } from "vitest";

import { createSpacePlacement } from "../../canvas/navigation-requests";
import { createInspectorActions } from "../inspector-actions";
import { createSpaceState } from "../../session/session-state";

function setup() {
  const state = createSpaceState();
  const placement = createSpacePlacement({
    insertionPoint: state.insertionPoint,
    revealRequest: state.revealRequest,
    viewportRequest: state.viewportRequest,
    zoomRequest: state.zoomRequest,
    selectionClearNonce: state.selectionClearNonce,
    sceneCue: state.sceneCue,
  });
  const requestMediaSeek = vi.fn<(nodeId: string, timeMs: number) => void>();
  return {
    state,
    actions: createInspectorActions(state, placement, requestMediaSeek),
    requestMediaSeek,
  };
}

describe("Space context dock actions", () => {
  it("opens Ask with an explicit contextual scope request", () => {
    const { state, actions } = setup();
    actions.requestAsk("selection");
    expect(state.inspectorMode.value).toBe("ask");
    expect(state.askScopeRequest.value).toMatchObject({ scope: "selection", restart: false });
  });

  it("returns a citation source to the previous Ask mode", () => {
    const { state, actions } = setup();
    actions.setInspectorMode("ask");
    actions.openSource(
      {
        nodeId: "note-1",
        nodeTitle: "Research",
        locator: {
          sourceRevision: "1",
          location: { kind: "note" },
        },
        text: "Evidence",
      },
      { reveal: false },
    );
    expect(state.inspectorMode.value).toBe("source");
    actions.closeSource();
    expect(state.inspectorMode.value).toBe("ask");
  });

  it("returns inspector evidence to Inspect mode and seeks media evidence", () => {
    const { state, actions, requestMediaSeek } = setup();
    actions.openSource(
      {
        nodeId: "video-1",
        nodeTitle: "Interview",
        locator: {
          sourceRevision: "1",
          location: { kind: "media", startMs: 1200, endMs: 2400 },
        },
      },
      { reveal: false },
    );
    expect(requestMediaSeek).toHaveBeenCalledWith("video-1", 1200);
    actions.closeSource();
    expect(state.inspectorMode.value).toBe("inspect");
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
