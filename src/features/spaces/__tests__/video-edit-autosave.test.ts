import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import { createSpaceNode } from "../canvas/node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import { createVideoEditProject } from "../video-editing/project";
import { useVideoProjectSave } from "../video-editing/use-video-project-save";
import { useVideoSessionStore } from "../video-editing/video-session-store";
import { flushSpaceSessionCommits } from "../workspace/session-participants";

const store = vi.hoisted(() => ({
  saveNodeContentForSpace:
    vi.fn<(spaceId: string, change: NodePayloadChangeInput) => Promise<SpaceNodePayload>>(),
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => store }));

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  store.saveNodeContentForSpace.mockImplementation(async (_spaceId, change) => change.data);
});

function openVideoEdit() {
  const source = createSpaceNode(
    { kind: "video", title: "Video", fileName: "v.mp4", assetPath: "/v.mp4", sourceRevision: "r" },
    { x: 0, y: 0 },
    "video",
  );
  const sessions = useVideoSessionStore();
  sessions.open({ id: "video-tab", kind: "video", spaceId: "origin", nodeId: source.id }, source);
  const session = sessions.sessions.get("video-tab")!;
  const original = createVideoEditProject({
    nodeId: source.id,
    sourceRevision: "r",
    durationMs: 2_000,
    sourceWidth: 640,
    sourceHeight: 360,
  });
  session.project.value = original;
  session.history.reset(original);
  let autosave!: ReturnType<typeof useVideoProjectSave>;
  const wrapper = mount(
    {
      setup() {
        autosave = useVideoProjectSave(session, shallowRef(true));
        return () => null;
      },
    },
    { global: { plugins: [i18n] } },
  );
  onTestFinished(() => wrapper.unmount());
  const edited = { ...original, output: { ...original.output, frameRate: 25 as const } };
  return { session, autosave, edited };
}

it("stores a video edit by itself shortly after the last change", async () => {
  vi.useFakeTimers();
  onTestFinished(() => {
    vi.useRealTimers();
  });
  const { session, autosave, edited } = openVideoEdit();
  session.history.commit(edited);
  expect(autosave.status.value).toBe("unsaved");

  await vi.advanceTimersByTimeAsync(1000);

  expect(store.saveNodeContentForSpace).toHaveBeenCalledOnce();
  expect(store.saveNodeContentForSpace.mock.calls[0]![1].data).toMatchObject({ videoEditProject: edited });
  expect(session.history.dirty.value).toBe(false);
  expect(autosave.status.value).toBe("saved");
});

it("stores what remains when its Space is left, without asking", async () => {
  const { session, edited } = openVideoEdit();
  session.history.commit(edited);

  await flushSpaceSessionCommits(undefined, { spaceIds: ["origin"] });

  expect(store.saveNodeContentForSpace).toHaveBeenCalledOnce();
  expect(session.node.value.data.videoEditProject).toEqual(edited);
  expect(session.history.dirty.value).toBe(false);
});
