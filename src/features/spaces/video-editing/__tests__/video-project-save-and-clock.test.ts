import { describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

const loadProjectionMock = vi.fn<(spaceId: string) => Promise<{ nodes: unknown[] }>>();
const saveContentMock = vi.fn<(spaceId: string, input: unknown) => Promise<unknown>>();
const activeSpaceId = shallowRef("space-1");
const storeNodes = shallowRef<unknown[]>([]);

vi.mock("vue-i18n", () => ({
  useI18n: () => ({ t: (key: string) => key }),
}));
vi.mock("vue-sonner", () => ({
  toast: { error: vi.fn<(message: string) => void>() },
}));
vi.mock("../../session/space-projection", () => ({
  loadSpaceSessionProjection: (spaceId: string) => loadProjectionMock(spaceId),
}));
vi.mock("../../session/space-store", () => ({
  useSpaceStore: () => ({
    get activeSpaceId() {
      return activeSpaceId.value;
    },
    get nodes() {
      return storeNodes.value;
    },
    saveNodeContentForSpace: (spaceId: string, input: unknown) => saveContentMock(spaceId, input),
  }),
}));

import { createPlaybackClock } from "../playback/playback-clock";
import { createVideoEditProject, type VideoEditProject } from "../project";
import { visibleTicks } from "../timeline/scale";
import { useVideoProjectSave } from "../use-video-project-save";
import type { VideoSession } from "../video-session-store";

function makeProject(): VideoEditProject {
  return createVideoEditProject({
    nodeId: "node-1",
    sourceRevision: "rev-1",
    durationMs: 10_000,
    sourceWidth: 1920,
    sourceHeight: 1080,
    frameRate: 30,
  });
}

function makeVideoNode(title: string, videoEditProject?: VideoEditProject) {
  return {
    id: "node-1",
    position: { x: 0, y: 0 },
    dimensions: { width: 320, height: 180 },
    data: {
      kind: "video",
      title,
      sourceRevision: "rev-1",
      assetPath: "assets/clip.mp4",
      mimeType: "video/mp4",
      durationMs: 10_000,
      width: 1920,
      height: 1080,
      videoEditProject,
    },
  };
}

function makeSession(
  initialNode: ReturnType<typeof makeVideoNode>,
  initialProject: VideoEditProject,
): VideoSession {
  const dirty = shallowRef(true);
  const project = shallowRef<VideoEditProject | undefined>(initialProject);
  return {
    tab: {
      id: "tab-1",
      spaceId: "space-1",
      nodeId: "node-1",
      kind: "video",
    } as unknown as VideoSession["tab"],
    node: shallowRef(initialNode) as unknown as VideoSession["node"],
    project,
    saving: shallowRef(false),
    error: shallowRef(),
    history: {
      dirty,
      markSaved: vi.fn<() => void>(() => {
        dirty.value = false;
      }),
    } as unknown as VideoSession["history"],
  } as unknown as VideoSession;
}

describe("playback clock and timeline bounds", () => {
  it("clamps seek positions to [0, durationMs] so the playhead never exceeds the timeline", () => {
    const clock = createPlaybackClock({
      context: () => ({ currentTime: 0, resume: vi.fn<() => Promise<void>>() }) as unknown as AudioContext,
      durationMs: () => 10_000,
    });
    clock.seek(25_000);
    expect(clock.positionMs()).toBe(10_000);
    clock.seek(-500);
    expect(clock.positionMs()).toBe(0);
  });

  it("extends visible ruler ticks across the viewport beyond short project durations", () => {
    const ticks = visibleTicks({ pxPerSecond: 100, scrollLeft: 0, viewportWidth: 1200, durationMs: 5_000 });
    expect(ticks.at(-1)).toBeGreaterThanOrEqual(12_000);
  });
});

describe("useVideoProjectSave conflict recovery", () => {
  it("rebases seamlessly when background metadata updated the node without changing videoEditProject", async () => {
    const project = makeProject();
    const initialNode = makeVideoNode("Old title");
    const freshNode = makeVideoNode("Updated metadata title");
    storeNodes.value = [];
    saveContentMock
      .mockRejectedValueOnce(new Error("Space node node-1 changed before this edit was saved"))
      .mockResolvedValueOnce({ ...freshNode.data, videoEditProject: project });
    loadProjectionMock.mockResolvedValueOnce({ nodes: [freshNode] });

    const scope = effectScope();
    const session = makeSession(initialNode, project);
    const active = shallowRef(true);
    const saver = scope.run(() => useVideoProjectSave(session, active))!;

    await saver.persist();

    expect(loadProjectionMock).toHaveBeenCalledWith("space-1");
    expect(saveContentMock).toHaveBeenCalledTimes(2);
    expect(session.error.value).toBeUndefined();
    expect(session.history.dirty.value).toBe(false);
    scope.stop();
  });

  it("sets conflict error when another edit actually diverged videoEditProject", async () => {
    const project = makeProject();
    const otherProject = { ...makeProject(), output: { ...project.output, width: 1280, height: 720 } };
    const initialNode = makeVideoNode("Title", project);
    const divergedNode = makeVideoNode("Title", otherProject);
    storeNodes.value = [];
    saveContentMock.mockRejectedValueOnce(new Error("Space node node-1 changed before this edit was saved"));
    loadProjectionMock.mockResolvedValueOnce({ nodes: [divergedNode] });

    const scope = effectScope();
    const session = makeSession(initialNode, project);
    const active = shallowRef(true);
    const saver = scope.run(() => useVideoProjectSave(session, active))!;

    await expect(saver.persist()).rejects.toThrow("changed before this edit was saved");
    expect(session.error.value).toBe("conflict");
    scope.stop();
  });
});
