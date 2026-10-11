import { mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { createSpaceNode } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useVideoEditActions } from "../presentation/video-editor/use-video-edit-actions";
import { useVideoEditHistory } from "../presentation/video-editor/use-video-edit-history";
import { useVideoEditSelection } from "../presentation/video-editor/use-video-edit-selection";
import { useVideoProjectPlayback } from "../presentation/video-editor/use-video-project-playback";
import { createVideoEditProject, type VideoEditProject } from "../video-editing/project";
import { deferred } from "./support/canvas-fixtures";

const context = vi.hoisted(() => ({
  store: {
    activeSpaceId: "origin",
    videoEditorNodeId: "source",
    updateNodeContents: vi.fn<() => Promise<void>>(),
    closeVideoEditor: vi.fn<() => void>(),
  },
  save: vi.fn<() => Promise<void>>(),
  render: vi.fn<(source: SpaceCanvasNode<SpaceVideoPayload>, project: VideoEditProject) => Promise<void>>(),
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.store }));
vi.mock("../video-editing/save", () => ({ saveVideoEditProject: context.save }));
vi.mock("../video-editing/render", () => ({ renderVideoEdit: context.render }));
vi.mock("vue-sonner", () => ({
  toast: { success: vi.fn<(message: string) => void>(), error: vi.fn<(message: string) => void>() },
}));

let wrapper: ReturnType<typeof mount> | undefined;
beforeEach(() => {
  vi.clearAllMocks();
  context.store.activeSpaceId = "origin";
  context.store.videoEditorNodeId = "source";
});
afterEach(() => wrapper?.unmount());

it.each(["space", "video", "project"])(
  "retains the render target when %s changes during save",
  async (change) => {
    const saving = deferred<void>();
    context.save.mockReturnValueOnce(saving.promise);
    const source = createSpaceNode<SpaceVideoPayload>(
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
    const originalProject = createVideoEditProject({
      sourceRevision: "revision",
      durationMs: 2000,
      sourceWidth: 640,
      sourceHeight: 480,
    });
    const node = shallowRef<SpaceCanvasNode>(source);
    const project = shallowRef<VideoEditProject | undefined>(originalProject);
    let actions!: ReturnType<typeof useVideoEditActions>;
    wrapper = mount(
      {
        setup() {
          const video = shallowRef<HTMLVideoElement | null>(null);
          actions = useVideoEditActions({
            node,
            data: shallowRef(source.data),
            project,
            video,
            history: useVideoEditHistory(project),
            selection: useVideoEditSelection(project),
            playback: useVideoProjectPlayback(video, project),
          });
          return () => null;
        },
      },
      {
        global: {
          plugins: [
            createI18n({
              legacy: false,
              locale: "en",
              messages: { en: { spaces: { media: { videoEditor: { project: { saved: "Saved" } } } } } },
            }),
          ],
        },
      },
    );
    const rendering = actions.render();
    await vi.waitFor(() => expect(context.save).toHaveBeenCalledOnce());
    if (change === "space") context.store.activeSpaceId = "other";
    else if (change === "video") {
      node.value = { ...source, id: "other" };
      context.store.videoEditorNodeId = "other";
    } else project.value = { ...originalProject, sourceRevision: "new-revision" };
    saving.resolve();
    await rendering;
    expect(context.render.mock.calls).toEqual(change === "project" ? [[source, originalProject]] : []);
    expect(actions.rendering.value).toBe(false);
  },
);
