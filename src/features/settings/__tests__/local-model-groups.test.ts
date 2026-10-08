import { expect, it } from "vitest";

import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import { localModelGroups } from "../local-model-groups";

function model(
  id: LocalModelAvailability["id"],
  capability: string,
  group: LocalModelAvailability["group"] = "audio",
): LocalModelAvailability {
  return {
    id,
    capability,
    group,
    name: id,
    operations: [],
    referenceTranscript: "unused",
    essential: false,
    selected: false,
    selectable: false,
    fixed: false,
    totalBytes: 1,
    installedBytes: 0,
    installed: false,
    verified: false,
    downloadAvailable: true,
    unavailableReason: null,
  };
}

it("separates images, audio and fixed knowledge resources independent of catalog order", () => {
  const entries = [
    model("u2netp", "imageSegmentation", "images"),
    model("u2net", "imageSegmentation", "images"),
    model("mossNano", "speechSynthesis"),
    model("qwen3Embedding", "embedding", "knowledge"),
    model("whisperSmall", "transcription"),
  ];
  const groups = localModelGroups(entries);
  expect(groups.map(({ id }) => id)).toEqual(["knowledge", "audio", "images"]);
  expect(groups[1]!.capabilities.map(({ id }) => id)).toEqual(["speechSynthesis", "transcription"]);
  expect(groups[2]!.capabilities[0]!.models.map(({ id }) => id)).toEqual(["u2netp", "u2net"]);
  expect(groups.flatMap(({ capabilities }) => capabilities.flatMap(({ models }) => models))).toHaveLength(5);
});

it("omits empty groups without manufacturing miscellaneous model entries", () => {
  expect(localModelGroups([])).toEqual([]);
  expect(localModelGroups([model("u2netp", "imageSegmentation", "images")]).map(({ id }) => id)).toEqual([
    "images",
  ]);
});
