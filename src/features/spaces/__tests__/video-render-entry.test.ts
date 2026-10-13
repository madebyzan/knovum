import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { nextTick, shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { createSpaceNode } from "../canvas/node";
import { videoEditCommands } from "../commands/video-edit-commands";
import { useVideoEditActions } from "../video-editing/presentation/use-video-edit-actions";
import { useVideoSelection } from "../video-editing/presentation/use-video-selection";
import { createVideoEditProject } from "../video-editing/project";
import type { VideoSelection } from "../video-editing/selection";
import { useVideoSessionStore } from "../video-editing/video-session-store";
import { deferred } from "./support/canvas-fixtures";

const context = vi.hoisted(() => ({
  store: { activeSpaceId: "origin", closeTab: vi.fn<() => Promise<void>>() },
  save: vi.fn<() => Promise<boolean>>(),
  render: vi.fn<(...args: unknown[]) => Promise<void>>(),
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.store }));
vi.mock("../video-editing/use-video-project-save", () => ({
  useVideoProjectSave: () => context.save,
}));
vi.mock("../video-editing/render", () => ({ renderVideoEdit: context.render }));
vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("vue-sonner", () => ({
  toast: { success: vi.fn<() => void>(), error: vi.fn<() => void>() },
}));
let wrapper: ReturnType<typeof mount> | undefined;
beforeEach(() => {
  vi.clearAllMocks();
  setActivePinia(createPinia());
});
afterEach(() => wrapper?.unmount());

function setup() {
  const source = createSpaceNode(
    {
      kind: "video",
      title: "Video",
      fileName: "video.mp4",
      assetPath: "/video.mp4",
      sourceRevision: "revision",
    },
    { x: 0, y: 0 },
    "source",
  );
  const sessions = useVideoSessionStore();
  sessions.open({ id: "tab", kind: "video", spaceId: "origin", nodeId: source.id }, source);
  const session = sessions.sessions.get("tab")!;
  const originalProject = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 2_000,
    sourceWidth: 640,
    sourceHeight: 480,
  });
  session.project.value = originalProject;
  session.history.reset(originalProject);
  const active = shallowRef(true);
  const playheadMs = shallowRef(0);
  let actions!: ReturnType<typeof useVideoEditActions>;
  wrapper = mount(
    {
      setup() {
        const selection = useVideoSelection({
          project: session.project,
          selection: session.selection,
          seek: (timeMs) => (playheadMs.value = timeMs),
          pause: vi.fn<() => void>(),
          playheadMs: () => playheadMs.value,
        });
        actions = useVideoEditActions({
          session,
          active,
          project: session.project,
          commit: (next, mergeKey) => session.history.commit(next, mergeKey),
          selection,
          playheadMs: () => playheadMs.value,
          seek: (timeMs) => (playheadMs.value = timeMs),
          pause: vi.fn<() => void>(),
          toggle: vi.fn<() => void>(),
          focusText: vi.fn<() => void>(),
        });
        return () => null;
      },
    },
    { global: { plugins: [createI18n({ legacy: false, locale: "en", messages: { en: {} } })] } },
  );
  return { source, session, originalProject, actions, active, playheadMs };
}

function runCommand(id: string) {
  videoEditCommands()
    .find((command) => command.id === id)!
    .run();
}

it.each(["space", "project"])(
  "keeps the admitted source snapshot when %s changes during save",
  async (change) => {
    const saving = deferred<boolean>();
    context.save.mockReturnValueOnce(saving.promise);
    const { source, session, originalProject, actions } = setup();
    session.history.commit({ ...originalProject, output: { ...originalProject.output, frameRate: 25 } });
    const rendering = actions.render();
    await vi.waitFor(() => expect(context.save).toHaveBeenCalledOnce());
    if (change === "space") context.store.activeSpaceId = "other";
    else session.project.value = { ...originalProject, sourceRevision: "new-revision" };
    saving.resolve(true);
    await rendering;
    expect(context.render.mock.calls).toEqual([
      ["origin", source, { ...originalProject, output: { ...originalProject.output, frameRate: 25 } }],
    ]);
    expect(actions.rendering.value).toBe(false);
  },
);

it("unregisters hidden video commands and restores the owning history on activation", async () => {
  const { session, active, playheadMs } = setup();
  playheadMs.value = 1_000;
  active.value = false;
  await nextTick();
  runCommand("video-edit.split");
  expect(session.project.value!.videoTrack.clips).toHaveLength(1);
  active.value = true;
  await nextTick();
  runCommand("video-edit.split");
  expect(session.project.value!.videoTrack.clips).toHaveLength(2);
  expect(session.selection.value).toMatchObject({ kind: "clip" });
  active.value = false;
  await nextTick();
  runCommand("video-edit.undo");
  expect(session.project.value!.videoTrack.clips).toHaveLength(2);
});

it("selects the overlay it adds and reports what the palette may run", async () => {
  const { session, actions, playheadMs } = setup();
  playheadMs.value = 500;
  await actions.addText();
  const selection = session.selection.value as VideoSelection;
  expect(selection).toMatchObject({ kind: "text" });
  expect(session.project.value!.textTrack.items).toHaveLength(1);
  expect(actions.canDelete.value).toBe(true);
  expect(actions.canSplit.value).toBe(true);
});
