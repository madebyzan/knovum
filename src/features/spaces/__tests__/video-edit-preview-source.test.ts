import { expect, it, onTestFinished, vi } from "vitest";
import { effectScope, nextTick, shallowRef } from "vue";
import type * as VueI18n from "vue-i18n";

import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useVideoEditPreviewSource } from "../presentation/video-editor/use-video-edit-preview-source";

vi.mock("vue-i18n", async (original) => ({
  ...(await original<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

function setup() {
  const data = shallowRef<SpaceVideoPayload | undefined>({
    kind: "video",
    title: "Interview",
    assetPath: "/spaces/interview.mp4",
    fileName: "interview.mp4",
    sourceRevision: "r1",
  });
  const proxyPath = shallowRef<string>();
  const playing = shallowRef(false);
  const scope = effectScope();
  const preview = scope.run(() => useVideoEditPreviewSource({ data, proxyPath, playing }))!;
  onTestFinished(() => scope.stop());
  return { data, proxyPath, playing, preview };
}

it("owns preview failures and clears them when the source revision changes", async () => {
  const { data, proxyPath, preview } = setup();
  proxyPath.value = "/cache/proxy.mp4";
  await nextTick();
  preview.onMetadataError();
  expect(preview.error.value).toBe("spaces.media.videoEditor.preview.unavailable");
  data.value = { ...data.value!, sourceRevision: "r2" };
  await nextTick();
  expect(preview.error.value).toBeUndefined();
  expect(preview.previewPath.value).toBe(data.value.assetPath);
});

it("switches a failed original preview to the prepared proxy before reporting a decoding failure", async () => {
  const { proxyPath, playing, preview } = setup();
  playing.value = true;
  proxyPath.value = "/cache/proxy.mp4";
  await nextTick();
  preview.onMetadataError();
  expect(preview.previewPath.value).toBe(proxyPath.value);
  expect(preview.error.value).toBeUndefined();
  preview.onMetadataError();
  expect(preview.error.value).toBe("spaces.media.videoEditor.preview.unavailable");
});
