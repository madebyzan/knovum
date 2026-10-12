import { expect, it } from "vitest";

import { removeClip } from "../clip-edits";
import { appendAsset } from "../material-edits";
import { createVideoEditProject } from "../project";

it("releases unused secondary revisions so deleted clips can be re-added from a changed node", () => {
  const initial = createVideoEditProject({
    nodeId: "primary",
    sourceRevision: "primary-revision",
    durationMs: 4000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  const material = {
    nodeId: "secondary",
    sourceRevision: "old",
    kind: "video" as const,
    durationMs: 3000,
    width: 1280,
    height: 720,
  };
  const composed = appendAsset(initial, material);
  const removed = removeClip(composed, composed.videoTrack.clips[1]!.id);
  expect(removed.assets.map(({ nodeId }) => nodeId)).toEqual(["primary"]);
  const restored = appendAsset(removed, { ...material, sourceRevision: "new" });
  expect(restored.assets[1]!.sourceRevision).toBe("new");
});

it("retains owning source identity even after removing its visual clip", () => {
  const initial = createVideoEditProject({
    nodeId: "primary",
    sourceRevision: "revision",
    durationMs: 4000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  const composed = appendAsset(initial, {
    nodeId: "image",
    sourceRevision: "image-revision",
    kind: "image",
    durationMs: 0,
    width: 720,
    height: 720,
  });
  const removed = removeClip(composed, composed.videoTrack.clips[0]!.id);
  expect(removed.assets[0]!.nodeId).toBe("primary");
  expect(removed.videoTrack.clips[0]!.assetId).toBe("image");
});
