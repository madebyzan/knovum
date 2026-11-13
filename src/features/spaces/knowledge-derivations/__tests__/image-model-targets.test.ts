import { describe, expect, it } from "vitest";

import { imageTargetOption, localModelAvailability } from "../../__tests__/support/image-operation-fixtures";
import { verifiedImageTargetOptions } from "../image-model-targets";

describe("verifiedImageTargetOptions", () => {
  it("keeps only ready targets whose download is verified", () => {
    const targets = [
      imageTargetOption("birefnetLite", "imageSegmentation", { selected: true }),
      imageTargetOption("ben2Base", "imageSegmentation"),
      imageTargetOption("birefnetGeneral", "imageSegmentation", {
        availability: { state: "blocked", reason: "configuration_incomplete" },
      }),
      imageTargetOption("lama", "imageInpainting"),
    ];
    const models = [
      localModelAvailability("birefnetLite", "imageSegmentation", { selected: true }),
      localModelAvailability("ben2Base", "imageSegmentation", { verified: false, installed: false }),
      localModelAvailability("birefnetGeneral", "imageSegmentation"),
      localModelAvailability("lama", "imageInpainting"),
    ];
    expect(verifiedImageTargetOptions(targets, models, "imageSegmentation")).toEqual([targets[0]]);
  });

  it("returns nothing when no download is verified", () => {
    const targets = [imageTargetOption("mobileSam", "imageSelection")];
    const models = [localModelAvailability("mobileSam", "imageSelection", { verified: false })];
    expect(verifiedImageTargetOptions(targets, models, "imageSelection")).toEqual([]);
  });
});
