import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { expect, it, onTestFinished, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import { createSpaceNode } from "../canvas/node";
import { createContextDock } from "../canvas/use-context-dock";
import { registerDocumentEditCommands } from "../commands/document-edit-commands";
import { useSpaceCommands } from "../commands/use-space-commands";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { useSpaceStore } from "../session/space-store";
import { registerSpaceSessionParticipant } from "../workspace/session-participants";
import { deferred } from "./support/canvas-fixtures";

vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("../enrichment/use-enrichment-job-observation", () => ({ useEnrichmentJobObservation: () => {} }));

it("routes commands and contextual panes to the active work surface", async () => {
  const pinia = createPinia();
  setActivePinia(pinia);
  const store = useSpaceStore();
  store.activeSpaceId = "inbox";
  store.nodes = [
    createSpaceNode(createSpaceNotePayload("Note"), { x: 0, y: 0 }, "note"),
    createSpaceNode(
      { kind: "video", title: "Video", fileName: "a.mp4", assetPath: "/a.mp4" },
      { x: 0, y: 0 },
      "video",
    ),
  ];
  const undo = vi.fn<() => void>();
  onTestFinished(
    registerDocumentEditCommands({
      canUndo: () => true,
      canRedo: () => false,
      undo,
      redo: () => {},
      find: () => {},
      hasOutline: () => true,
      focusOutline: () => {},
      save: async () => {},
      close: async () => {},
    }),
  );
  const openPane = vi.fn<() => void>();
  const dock = createContextDock({ open: openPane, close: () => {} });
  let commands!: ReturnType<typeof useSpaceCommands>;
  const wrapper = mount(
    {
      setup() {
        commands = useSpaceCommands({
          sidebar: { reveal: () => {} },
          rightSidebar: { open: () => {}, close: () => {} },
        });
        return () => null;
      },
    },
    { global: { plugins: [pinia, i18n] } },
  );
  onTestFinished(() => wrapper.unmount());
  expect(commands.value.map(({ id }) => id)).toContain("spaces.zoom-in");
  expect(store.workbench.outline).toBe(true);
  const save = deferred<void>();
  onTestFinished(registerSpaceSessionParticipant({ commit: () => save.promise }));
  const switching = store.openDocumentEditor("note");
  expect(commands.value.map(({ id }) => id)).not.toContain("spaces.zoom-in");
  save.resolve();
  await switching;
  expect(store.workbench.rightSidebar).toBe(false);
  expect(store.workbench.outline).toBe(false);
  expect(commands.value.map(({ id }) => id)).not.toContain("spaces.zoom-in");
  commands.value.find(({ id }) => id === "document.undo")!.run();
  expect(undo).toHaveBeenCalledOnce();
  dock.openSource({
    nodeId: "note",
    nodeTitle: "Note",
    locator: { sourceRevision: "1", location: { kind: "note" } },
  });
  expect(store.sourcePeek?.nodeId).toBe("note");
  expect(openPane).not.toHaveBeenCalled();
  expect(store.revealRequest).toBeUndefined();
  await store.openVideoEditor("video");
  expect(store.sourcePeek).toBeUndefined();
  expect(commands.value.map(({ id }) => id)).toContain("video-edit.undo");
  expect(commands.value.map(({ id }) => id)).not.toContain("document.undo");
  expect(store.workbench.rightSidebar).toBe(false);
  await store.openSpace("inbox");
  expect(store.surface.kind).toBe("video");
  await store.returnToCanvas();
  expect(store.surface.kind).toBe("canvas");
  expect(store.workbench.outline).toBe(true);
});
