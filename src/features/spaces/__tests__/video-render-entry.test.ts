import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { nextTick, shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { createSpaceNode } from "../canvas/node";
import { videoEditCommands } from "../commands/video-edit-commands";
import { useVideoEditActions } from "../video-editing/presentation/use-video-edit-actions";
import { useVideoEditSelection } from "../video-editing/presentation/use-video-edit-selection";
import { useVideoProjectPlayback } from "../video-editing/presentation/use-video-project-playback";
import { createVideoEditProject } from "../video-editing/project";
import { useVideoSessionStore } from "../video-editing/video-session-store";
import { deferred } from "./support/canvas-fixtures";

const context = vi.hoisted(() => ({
  store: { activeSpaceId: "origin", closeEditorTab: vi.fn<() => Promise<void>>() },
  save: vi.fn<() => Promise<boolean>>(),
  render: vi.fn<(...args: unknown[]) => Promise<void>>(),
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.store }));
vi.mock("../video-editing/use-video-project-save", () => ({ useVideoProjectSave: () => context.save }));
vi.mock("../video-editing/render", () => ({ renderVideoEdit: context.render }));
vi.mock("vue-sonner", () => ({ toast: { success: vi.fn<() => void>(), error: vi.fn<() => void>() } }));
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
    durationMs: 2000,
    sourceWidth: 640,
    sourceHeight: 480,
  });
  session.project.value = originalProject;
  const active = shallowRef(true);
  let actions!: ReturnType<typeof useVideoEditActions>;
  wrapper = mount(
    {
      setup() {
        actions = useVideoEditActions({
          session,
          active,
          node: session.node,
          project: session.project,
          history: session.history,
          selection: useVideoEditSelection(session.project),
          playback: useVideoProjectPlayback(session.project, active, session.timelineMs),
        });
        return () => null;
      },
    },
    { global: { plugins: [createI18n({ legacy: false, locale: "en", messages: { en: {} } })] } },
  );
  return { source, session, originalProject, actions, active };
}

it.each(["space", "project"])(
  "keeps the admitted source snapshot when %s changes during save",
  async (change) => {
    const saving = deferred<boolean>();
    context.save.mockReturnValueOnce(saving.promise);
    const { source, session, originalProject, actions } = setup();
    const rendering = actions.render();
    await vi.waitFor(() => expect(context.save).toHaveBeenCalledOnce());
    if (change === "space") context.store.activeSpaceId = "other";
    else session.project.value = { ...originalProject, sourceRevision: "new-revision" };
    saving.resolve(true);
    await rendering;
    expect(context.render.mock.calls).toEqual([["origin", source, originalProject]]);
    expect(actions.rendering.value).toBe(false);
  },
);

it("unregisters hidden video commands and restores the owning history on activation", async () => {
  const { session, active } = setup();
  session.timelineMs.value = 1000;
  active.value = false;
  await nextTick();
  videoEditCommands()
    .find(({ id }) => id === "video-edit.split")!
    .run();
  expect(session.project.value!.videoTrack.clips).toHaveLength(1);
  active.value = true;
  await nextTick();
  videoEditCommands()
    .find(({ id }) => id === "video-edit.split")!
    .run();
  expect(session.project.value!.videoTrack.clips).toHaveLength(2);
  active.value = false;
  await nextTick();
  videoEditCommands()
    .find(({ id }) => id === "video-edit.undo")!
    .run();
  expect(session.project.value!.videoTrack.clips).toHaveLength(2);
});
