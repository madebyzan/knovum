import { flushPromises } from "@vue/test-utils";
import { expect, it } from "vitest";

import { mocks, render } from "./support/local-model-controls";

it("downloads the selected model through capability preparation without account forms", async () => {
  mocks.prepare.mockReturnValue({ completion: Promise.resolve(), cancel: mocks.cancel });
  const wrapper = render();
  expect(wrapper.find("input").exists()).toBe(false);
  await wrapper.get('[data-testid="model-download"]').trigger("click");
  await flushPromises();
  expect(wrapper.emitted("preparationChange")).toEqual([[true], [false]]);
  expect(mocks.prepare).toHaveBeenCalledWith(
    { kind: "model", model: "sherpaDiarization" },
    "settings.ai.localModels.downloading",
    expect.any(Function),
  );
});

it("keeps preparation cancellable while active", async () => {
  let finish: (() => void) | undefined;
  mocks.prepare.mockReturnValue({
    completion: new Promise<void>((resolve) => {
      finish = resolve;
    }),
    cancel: mocks.cancel,
  });
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  await wrapper.get("button").trigger("click");
  expect(mocks.cancel).toHaveBeenCalledOnce();
  expect(wrapper.emitted("preparationChange")).toEqual([[true]]);
  finish?.();
  await flushPromises();
  expect(wrapper.emitted("preparationChange")).toEqual([[true], [false]]);
});

it("shows a corrupt optional installation with a safe removal action", async () => {
  const wrapper = render({ installed: true, installedBytes: 30 });
  expect(wrapper.text()).toContain("settings.ai.localModels.invalid");
  await wrapper.get('[data-testid="model-remove"]').trigger("click");
  await flushPromises();
  expect(mocks.remove).toHaveBeenCalledWith("sherpaDiarization");
  expect(mocks.invalidate).toHaveBeenCalledOnce();
});

it("repairs a corrupt optional snapshot directly without requiring removal first", async () => {
  mocks.prepare.mockReturnValue({ completion: Promise.resolve(), cancel: mocks.cancel });
  const wrapper = render({ installed: true, installedBytes: 30 });
  await wrapper.get('[data-testid="model-download"]').trigger("click");
  await flushPromises();
  expect(mocks.prepare).toHaveBeenCalledWith(
    { kind: "model", model: "sherpaDiarization" },
    "settings.ai.localModels.downloading",
    expect.any(Function),
  );
  expect(mocks.remove).not.toHaveBeenCalled();
});

it("keeps the embedding model fixed and essential models under application ownership", () => {
  const wrapper = render({
    id: "qwen3Embedding",
    essential: true,
    fixed: true,
    installed: true,
    verified: true,
  });
  expect(wrapper.text()).toContain("settings.ai.localModels.fixedEmbedding");
  expect(wrapper.find('[data-testid="model-remove"]').exists()).toBe(false);
});

it("allows a corrupt essential model to be downloaded again without exposing removal", async () => {
  mocks.prepare.mockReturnValue({ completion: Promise.resolve(), cancel: mocks.cancel });
  const wrapper = render({ id: "qwen3Embedding", essential: true, installed: true });
  expect(wrapper.find('[data-testid="model-remove"]').exists()).toBe(false);
  await wrapper.get('[data-testid="model-download"]').trigger("click");
  await flushPromises();
  expect(mocks.prepare).toHaveBeenCalledWith(
    { kind: "model", model: "qwen3Embedding" },
    "settings.ai.localModels.downloading",
    expect.any(Function),
  );
});

it.each([
  ["whisperTiny", "transcription"],
  ["voxCpm2", "speechSynthesis"],
  ["u2net", "imageSegmentation"],
  ["spleeter2StemsFull", "speechSeparation"],
  ["sherpaDiarizationFull", "speechDiarization"],
] as const)("selects a verified %s default without altering existing voices", async (id, capability) => {
  const wrapper = render({ id, capability, selectable: true, installed: true, verified: true });
  await wrapper.get('[data-testid="model-select"]').trigger("click");
  await flushPromises();
  expect(mocks.select).toHaveBeenCalledWith(id);
  await wrapper.setProps({
    model: {
      ...wrapper.props("model"),
      id,
      capability,
      selectable: true,
      installed: true,
      verified: true,
      selected: true,
    },
  });
  expect(wrapper.find('[data-testid="model-remove"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="model-select"]').exists()).toBe(false);
});

it("requires downloading an alternative before it can be selected", () => {
  const wrapper = render({ id: "whisperLargeV3", capability: "transcription" });
  expect(wrapper.find('[data-testid="model-select"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="model-download"]').exists()).toBe(true);
});
