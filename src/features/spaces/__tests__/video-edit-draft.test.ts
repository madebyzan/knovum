import { createPinia, setActivePinia } from "pinia";
import { expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";
import type * as VueI18n from "vue-i18n";

import { createSpaceNode } from "../canvas/node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { createVideoEditProject } from "../video-editing/project";
import { useVideoSessionStore } from "../video-editing/video-session-store";

vi.mock("vue-i18n", async (original) => ({
  ...(await original<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

it("opens a saved reactive video project as an independent draft", () => {
  const data = ref<SpaceVideoPayload>({
    kind: "video",
    title: "Video",
    fileName: "video.mp4",
    assetPath: "/owned/video.mp4",
    sourceRevision: "r1",
    videoEditProject: createVideoEditProject({
      sourceRevision: "r1",
      durationMs: 1000,
      sourceWidth: 1920,
      sourceHeight: 1080,
    }),
  });
  const scope = effectScope();
  try {
    setActivePinia(createPinia());
    const sessions = useVideoSessionStore();
    sessions.open(
      { id: "tab", kind: "video", spaceId: "space", nodeId: "video" },
      createSpaceNode(data.value, { x: 0, y: 0 }, "video"),
    );
    const editing = sessions.sessions.get("tab")!;
    expect(editing.project.value).toEqual(data.value.videoEditProject);
    editing.project.value!.videoTrack.clips[0]!.volume = 0;
    expect(data.value.videoEditProject!.videoTrack.clips[0]!.volume).toBe(1);
  } finally {
    scope.stop();
  }
});
