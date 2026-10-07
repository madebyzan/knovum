import { mount } from "@vue/test-utils";
import { beforeEach, expect, it, vi } from "vitest";

import type { LocalResourcesStatus } from "../../../shared/contracts/generated/LocalResourcesStatus";
import PythonRuntimeSettingsField from "../components/PythonRuntimeSettingsField.vue";

const mocks = vi.hoisted(() => ({
  data: { value: undefined as Pick<LocalResourcesStatus, "platform" | "pythonRuntimes"> | undefined },
  error: { value: null as Error | null },
}));
vi.mock("../../../foundation/runtime/resource-query", () => ({ useLocalResourcesQuery: () => mocks }));
vi.mock("../../../foundation/runtime/resource-preparation", () => ({
  startResourcePreparation: vi.fn<() => void>(),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

beforeEach(() => {
  mocks.data.value = {
    platform: "unsupported",
    pythonRuntimes: {
      resources: { state: "not_published" },
      "light-inference": { state: "not_published" },
      "torch-inference": { state: "not_published" },
      "voice-generation": { state: "not_published" },
      agent: { state: "not_published" },
    },
  };
  mocks.error.value = null;
});

function render() {
  return mount(PythonRuntimeSettingsField, {
    props: { profile: "torch-inference", label: "Inference" },
    global: {
      stubs: {
        PreferenceField: {
          props: ["description"],
          template: "<div><p>{{ description }}</p><slot /></div>",
        },
        UiButton: { props: ["disabled"], template: '<button :disabled="disabled"><slot /></button>' },
      },
    },
  });
}

it("distinguishes unsupported architecture from missing runtime publication", () => {
  const wrapper = render();
  try {
    expect(wrapper.text()).toContain("settings.ai.pythonRuntimes.unsupported");
    expect(wrapper.text()).not.toContain("settings.ai.pythonRuntimes.unavailable");
    expect(wrapper.find("button").attributes("disabled")).toBeDefined();
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  } finally {
    wrapper.unmount();
  }
});

it("reports missing publication on a supported architecture", () => {
  mocks.data.value!.platform = "macos-arm64";
  const wrapper = render();
  try {
    expect(wrapper.text()).toContain("settings.ai.pythonRuntimes.unavailable");
    expect(wrapper.text()).not.toContain("settings.ai.pythonRuntimes.unsupported");
  } finally {
    wrapper.unmount();
  }
});

it("keeps genuine resource inspection failures visible", () => {
  mocks.data.value = undefined;
  mocks.error.value = new Error("Unable to inspect runtime manifest");
  const wrapper = render();
  try {
    expect(wrapper.get('[role="alert"]').text()).toBe("Unable to inspect runtime manifest");
  } finally {
    wrapper.unmount();
  }
});

it("scopes a broken runtime to its own setting without hiding another runtime", () => {
  mocks.data.value!.platform = "macos-x64";
  mocks.data.value!.pythonRuntimes["torch-inference"] = {
    state: "invalid",
    message: "Inference manifest failed verification",
  };
  mocks.data.value!.pythonRuntimes.agent = { state: "ready" };
  const inference = render();
  const agent = mount(PythonRuntimeSettingsField, { props: { profile: "agent", label: "Agent" } });
  try {
    expect(inference.get('[role="alert"]').text()).toBe("Inference manifest failed verification");
    expect(agent.find('[role="alert"]').exists()).toBe(false);
    expect(agent.find("button").exists()).toBe(false);
  } finally {
    inference.unmount();
    agent.unmount();
  }
});
