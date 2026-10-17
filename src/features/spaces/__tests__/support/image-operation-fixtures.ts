import { createSpaceNode } from "../../canvas/node";
import type { ImageOperationCapabilities } from "../../contracts/generated/ImageOperationCapabilities";
import { decideCreationIntents } from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";

/** One selected image with every derivation capability withheld. */
export function selectedImageIntents() {
  return decideCreationIntents({
    selectionCount: 1,
    sourceCount: 1,
    imageCount: 1,
    canRemoveBackground: false,
    canSplitImageLayers: false,
    canAddVideoSubtitles: false,
  });
}

export function imageSourceNode() {
  const data: SpaceImagePayload = {
    kind: "image",
    title: "Image",
    fileName: "image.png",
    assetPath: "/managed/image.png",
    sourceRevision: "revision-1",
    intrinsicWidth: 1024,
    intrinsicHeight: 1024,
  };
  return { ...createSpaceNode(data, { x: 0, y: 0 }, "image"), data };
}

export function blockedImageCapabilities(): ImageOperationCapabilities {
  const blocked = { state: "blocked", reason: "configuration_incomplete" } as const;
  return {
    targets: [
      {
        execution: { type: "local", model: "migan", specId: "spec" },
        label: "MI-GAN",
        capability: "imageInpainting",
        availability: blocked,
        selected: true,
        maximumSide: null,
        maximumPixels: null,
      },
    ],
    remote: null,
    selection: blocked,
    depth: blocked,
    maskedRemoval: blocked,
    promptRemoval: blocked,
    upscale: blocked,
    multiAngle: blocked,
  };
}
