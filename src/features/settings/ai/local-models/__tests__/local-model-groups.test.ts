import { expect, it } from "vitest";

import type { LocalModelAvailability } from "../../../../../shared/contracts/generated/LocalModelAvailability";
import { buildLocalModelGroups } from "../groups";

function model(
  modelId: LocalModelAvailability["id"],
  capability: LocalModelAvailability["capability"],
  group: LocalModelAvailability["group"] = "audio",
): LocalModelAvailability {
  return {
    id: modelId,
    capability,
    group,
    name: modelId,
    operations: [],
    referenceTranscript: "unused",
    referenceDuration: null,
    essential: false,
    selected: false,
    selectable: false,
    fixed: false,
    totalBytes: 1,
    installedBytes: 0,
    installed: false,
    verified: false,
    memoryAdvisoryBytes: null,
  };
}

it("separates images, audio and fixed knowledge resources independent of catalog order", () => {
  const entries = [
    model("birefnetLite", "imageSegmentation", "images"),
    model("ben2Base", "imageSegmentation", "images"),
    model("mossNano", "speechSynthesis"),
    model("qwen3Embedding", "embedding", "knowledge"),
    model("whisperSmall", "transcription"),
  ];
  const groups = buildLocalModelGroups(entries);
  expect(groups.map(({ id }) => id)).toEqual(["knowledge", "audio", "images"]);
  expect(groups[1]!.capabilities.map(({ id }) => id)).toEqual(["speechSynthesis", "transcription"]);
  expect(groups[2]!.capabilities[0]!.models.map(({ id }) => id)).toEqual(["birefnetLite", "ben2Base"]);
  expect(groups.flatMap(({ capabilities }) => capabilities.flatMap(({ models }) => models))).toHaveLength(5);
});

it("omits empty groups without manufacturing miscellaneous model entries", () => {
  expect(buildLocalModelGroups([])).toEqual([]);
  expect(
    buildLocalModelGroups([model("birefnetLite", "imageSegmentation", "images")]).map(({ id }) => id),
  ).toEqual(["images"]);
});
