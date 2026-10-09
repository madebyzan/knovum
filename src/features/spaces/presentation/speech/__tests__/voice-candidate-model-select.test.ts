import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef, type ShallowRef } from "vue";

import type { LocalModelAvailability } from "../../../../../shared/contracts/generated/LocalModelAvailability";
import VoiceCandidateModelSelect from "../VoiceCandidateModelSelect.vue";

const mocks = vi.hoisted(() => ({
  models: undefined as ShallowRef<LocalModelAvailability[] | undefined> | undefined,
}));
vi.mock("../../../../../foundation/runtime/use-resource-queries", () => ({
  useLocalModelsQuery: () => ({ data: mocks.models, error: shallowRef(null), isPending: shallowRef(false) }),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

function entry(modelId: "mossNano" | "voxCpm2" | "voxCpm2Q8", selected: boolean): LocalModelAvailability {
  const vox = modelId !== "mossNano";
  return {
    id: modelId,
    selected,
    selectable: true,
    group: "audio",
    name: modelId,
    capability: "speechSynthesis",
    operations: vox ? ["voiceCloning", "voiceDesign"] : ["voiceCloning"],
    referenceTranscript: vox ? "optionalConfirmed" : "unused",
    essential: false,
    fixed: false,
    installed: true,
    verified: true,
    totalBytes: modelId === "voxCpm2" ? 500 : modelId === "voxCpm2Q8" ? 300 : 100,
    installedBytes: 100,
    downloadAvailable: true,
    unavailableReason: null,
  };
}
beforeEach(() => {
  mocks.models = shallowRef([entry("mossNano", false), entry("voxCpm2", true)]);
});
function render(sourceKind: "reference" | "designed" = "reference") {
  const wrapper = mount(VoiceCandidateModelSelect, {
    props: {
      sourceKind,
      disabled: false,
      modelValue: "mossNano",
      available: false,
      referenceTranscript: "unused",
    },
    global: { stubs: { UiSelect: true } },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("starts a new clone with the speech model selected in settings", () => {
  const wrapper = render();
  expect(wrapper.emitted("update:modelValue")).toEqual([["voxCpm2"]]);
});

it("waits for the catalog without replacing a choice when it refetches", async () => {
  mocks.models!.value = undefined;
  const wrapper = render();
  expect(wrapper.emitted("update:modelValue")).toBeUndefined();
  mocks.models!.value = [entry("mossNano", false), entry("voxCpm2", true)];
  await flushPromises();
  expect(wrapper.emitted("update:modelValue")).toEqual([["voxCpm2"]]);
  await wrapper.setProps({ modelValue: "mossNano" });
  mocks.models!.value = [...mocks.models!.value];
  await flushPromises();
  expect(wrapper.emitted("update:modelValue")).toEqual([["voxCpm2"]]);
});

it("uses the design-capable model for design and restores the cloning default", async () => {
  mocks.models!.value = [entry("mossNano", true), entry("voxCpm2", false), entry("voxCpm2Q8", false)];
  const wrapper = render("designed");
  expect(wrapper.emitted("update:modelValue")).toEqual([["voxCpm2Q8"]]);
  expect(wrapper.emitted("update:referenceTranscript")?.at(-1)).toEqual(["optionalConfirmed"]);
  await wrapper.setProps({ modelValue: "voxCpm2Q8", sourceKind: "reference" });
  expect(wrapper.emitted("update:modelValue")?.at(-1)).toEqual(["mossNano"]);
});

it("keeps the selected design model instead of replacing it with the smallest option", () => {
  mocks.models!.value = [entry("mossNano", false), entry("voxCpm2", true), entry("voxCpm2Q8", false)];
  const wrapper = render("designed");
  expect(wrapper.emitted("update:modelValue")).toEqual([["voxCpm2"]]);
});

it("selects a compatible model after a disabled form becomes editable", async () => {
  mocks.models!.value = [entry("mossNano", true), entry("voxCpm2Q8", false)];
  const wrapper = mount(VoiceCandidateModelSelect, {
    props: {
      sourceKind: "designed",
      disabled: true,
      modelValue: "mossNano",
      available: false,
      referenceTranscript: "unused",
    },
    global: { stubs: { UiSelect: true } },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.emitted("update:modelValue")).toBeUndefined();
  await wrapper.setProps({ disabled: false });
  expect(wrapper.emitted("update:modelValue")).toEqual([["voxCpm2Q8"]]);
});
