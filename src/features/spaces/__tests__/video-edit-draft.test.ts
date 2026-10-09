import { expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";

import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useVideoEditProject } from "../presentation/video-editor/use-video-edit-project";
import { createVideoEditProject } from "../video-editing/project";

vi.mock("vue-i18n", async (original) => ({
  ...(await original<typeof import("vue-i18n")>()),
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
    const editing = scope.run(() => useVideoEditProject(ref("video"), data, ref(undefined)))!;
    expect(editing.project.value).toEqual(data.value.videoEditProject);
    editing.project.value!.videoTrack.clips[0]!.volume = 0;
    expect(data.value.videoEditProject!.videoTrack.clips[0]!.volume).toBe(1);
  } finally {
    scope.stop();
  }
});
