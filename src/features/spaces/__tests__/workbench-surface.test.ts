import { flushPromises } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";

import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { showSpaceLibrary } from "../workspace/library-actions";
import { registerSpaceSessionParticipant } from "../workspace/session-participants";
import { openEditorTab } from "../workspace/workbench-navigation-actions";
import { activateTestEditor, createTestSessionState, createTestSpace } from "./support/session-context";

function setup() {
  const { state, context } = createTestSessionState({
    spaces: [createTestSpace({ thumbnailPath: "/cover.jpg" })],
  });
  state.thumbnailDirty.value = false;
  return { state, context };
}

it("waits for the editing session before showing the library", async () => {
  const { state, context } = setup();
  let accept!: () => void;
  onTestFinished(
    registerSpaceSessionParticipant({ commit: () => new Promise<void>((resolve) => (accept = resolve)) }),
  );
  const navigation = showSpaceLibrary(state, context, "all");
  await flushPromises();
  expect(state.surface.value.kind).toBe("canvas");
  accept();
  await navigation;
  expect(state.surface.value.kind).toBe("library");
});

it("preserves the current surface when an editor cancels library navigation", async () => {
  const { state, context } = setup();
  onTestFinished(
    registerSpaceSessionParticipant({
      commit: async () => {
        throw new ApplicationLeaveCancelled();
      },
    }),
  );
  await expect(Promise.resolve().then(() => showSpaceLibrary(state, context, "all"))).rejects.toBeInstanceOf(
    ApplicationLeaveCancelled,
  );
  expect(state.surface.value.kind).toBe("canvas");
});

it("keeps a video editor and its library filter after a failed save", async () => {
  const { state, context } = setup();
  activateTestEditor(context, "video", "video-1");
  state.libraryScope.value = "favorites";
  onTestFinished(
    registerSpaceSessionParticipant({
      commit: async () => {
        throw new Error("Save failed");
      },
    }),
  );
  await expect(showSpaceLibrary(state, context, "all")).rejects.toThrow("Save failed");
  expect(state.surface.value).toEqual({ kind: "video", nodeId: "video-1" });
  expect(state.libraryScope.value).toBe("favorites");
});

it.each(["retained", "removed"])("saves before replacing a document whose target is %s", async (target) => {
  const { state, context } = setup();
  activateTestEditor(context, "document", "first");
  state.nodes.value = ["first", "second"].map((id) =>
    createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id),
  );
  let accept!: () => void;
  onTestFinished(
    registerSpaceSessionParticipant({ commit: () => new Promise<void>((resolve) => (accept = resolve)) }),
  );
  const switching = openEditorTab(context, "document", "second");
  await flushPromises();
  expect(state.surface.value).toEqual({ kind: "document", nodeId: "first" });
  if (target === "removed") state.nodes.value = state.nodes.value.filter(({ id }) => id === "first");
  accept();
  const outcome = await switching.then(
    () => undefined,
    (error: unknown) => error,
  );
  expect(outcome).toEqual(target === "removed" ? new Error("Editor content is unavailable") : undefined);
  expect(state.surface.value).toEqual({
    kind: "document",
    nodeId: target === "removed" ? "first" : "second",
  });
});
