import { flushPromises, shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import { reactive, shallowReadonly } from "vue";
import type * as VueI18n from "vue-i18n";

import type { SavedCapturedWorkflow } from "../contracts/generated/SavedCapturedWorkflow";
import SpaceWorkflowEditForm from "../presentation/workflows/SpaceWorkflowEditForm.vue";
import SpaceWorkflowOutputEditor from "../presentation/workflows/SpaceWorkflowOutputEditor.vue";
import SpaceWorkflowParameterFields from "../presentation/workflows/SpaceWorkflowParameterFields.vue";
import type { reviseCapturedWorkflow } from "../workflows/captured-workflow-ipc";

const revise = vi.hoisted(() => vi.fn<typeof reviseCapturedWorkflow>());
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));
vi.mock("../workflows/captured-workflow-ipc", () => ({ reviseCapturedWorkflow: revise }));
vi.mock("../workflows/action-catalog", () => ({
  listSpaceActions: async () => [{ id: "generate_image", parametersSchema: {} }],
}));

function workflow(): SavedCapturedWorkflow {
  return {
    schemaVersion: 1,
    workflowId: "workflow",
    version: 1,
    name: "Images",
    sourceSpaceId: "space",
    sourceDigest: "digest",
    selectedNodeIds: ["image"],
    createdAt: 1,
    candidate: {
      inputSlots: [],
      steps: [
        {
          id: "step",
          actionId: "generate_image",
          actionVersion: 1,
          outputKind: "image",
          executionSpec: {
            parameters: { prompt: "Original", count: 1, transparent: false, options: { quality: "high" } },
            target: null,
          },
          inputs: [],
        },
      ],
      outputSlots: [{ id: "output", name: "Image", kind: "image", stepId: "step" }],
    },
  };
}

describe("workflow edit draft", () => {
  it("opens reactive query data and saves an independent plain draft", async () => {
    const input = workflow();
    input.candidate.steps[0]!.executionSpec.parameters.options = reactive({ quality: "high" });
    const source = reactive(input);
    revise.mockResolvedValue(workflow());
    const wrapper = shallowMount(SpaceWorkflowEditForm, { props: { workflow: shallowReadonly(source) } });
    await flushPromises();
    expect(wrapper.find("h3").text()).toBe("spaces.creation.workflow.edit");
    wrapper.findComponent(SpaceWorkflowParameterFields).vm.$emit("update-text", "step:prompt", "Edited");
    wrapper
      .findComponent(SpaceWorkflowParameterFields)
      .vm.$emit("update-boolean", "step", "transparent", true);
    wrapper
      .findComponent(SpaceWorkflowOutputEditor)
      .vm.$emit("update:modelValue", [
        ...reactive([{ id: "output", name: "Edited image", kind: "image", stepId: "step" }]),
      ]);
    await wrapper.find("form").trigger("submit");
    await flushPromises();
    const request = revise.mock.calls.at(-1)![0];
    expect(request.stepParameters.step).toEqual({
      prompt: "Edited",
      count: 1,
      transparent: true,
      options: { quality: "high" },
    });
    expect(request.outputSlots[0]!.name).toBe("Edited image");
    expect(() => structuredClone(request)).not.toThrow();
    expect(source.candidate.steps[0]!.executionSpec.parameters.prompt).toBe("Original");
    expect(source.candidate.outputSlots[0]!.name).toBe("Image");
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
    wrapper.unmount();
  });

  it("keeps the original version when query data refreshes during editing", async () => {
    const original = workflow();
    revise.mockResolvedValue(original);
    const wrapper = shallowMount(SpaceWorkflowEditForm, { props: { workflow: reactive(original) } });
    await flushPromises();
    const refreshed = workflow();
    refreshed.version = 2;
    refreshed.candidate.steps[0]!.executionSpec.parameters.prompt = "Concurrent edit";
    await wrapper.setProps({ workflow: shallowReadonly(reactive(refreshed)) });
    await wrapper.find("form").trigger("submit");
    await flushPromises();
    expect(revise).toHaveBeenLastCalledWith(
      expect.objectContaining({
        expectedVersion: 1,
        stepParameters: { step: original.candidate.steps[0]!.executionSpec.parameters },
      }),
    );
    wrapper.unmount();
  });
});
