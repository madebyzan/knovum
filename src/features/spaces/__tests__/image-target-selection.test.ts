import { expect, it } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import { selectedImageTarget } from "../knowledge-derivations/image-operation";
import { blockedImageCapabilities } from "./support/image-operation-fixtures";

it("uses the selected target rather than the first available model", () => {
  const chosen = blockedImageCapabilities().targets[0]!;
  chosen.availability = { state: "ready" };
  const alternative = { ...chosen, selected: false, label: "Alternative" };
  alternative.execution = { type: "local", model: "efficientSamTi", specId: "alternative" };
  expect(selectedImageTarget([alternative, chosen])).toEqual(chosen.execution);
});

it("rejects a blocked preference without switching to an available model", () => {
  const chosen = blockedImageCapabilities().targets[0]!;
  const alternative = { ...chosen, selected: false, availability: { state: "ready" as const } };
  expect(() => selectedImageTarget([chosen, alternative])).toThrow(
    i18n.global.t("spaces.media.imageEditor.selection.modelUnavailable"),
  );
});

it("requires an explicit preference even when one model is available", () => {
  const model = blockedImageCapabilities().targets[0]!;
  model.selected = false;
  model.availability = { state: "ready" };
  expect(() => selectedImageTarget([model])).toThrow(
    i18n.global.t("spaces.media.imageEditor.selection.modelUnavailable"),
  );
});
