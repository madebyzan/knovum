import { shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import { ref, type ComputedRef } from "vue";
import type * as VueI18n from "vue-i18n";

const captured = vi.hoisted(() => ({
  options: undefined as
    | ComputedRef<{ queryKey: readonly unknown[]; queryFn: () => Promise<unknown> }>
    | undefined,
  preview: vi
    .fn<(spaceId: string, nodeIds: readonly string[]) => Promise<undefined>>()
    .mockResolvedValue(undefined),
}));
vi.mock("@tanstack/vue-query", () => ({
  useQuery: (options: NonNullable<typeof captured.options>) => {
    captured.options = options;
    return { data: ref(undefined), isFetching: ref(false), error: ref(null) };
  },
}));
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));
vi.mock("../session/space-store", () => ({
  useSpaceStore: () => ({ activeSpaceId: "space-1", workflowCaptureSelection: ["output-1"], nodes: [] }),
}));
vi.mock("../workflows/captured-workflow-ipc", () => ({ previewWorkflowCapture: captured.preview }));
vi.mock("../workflows/presentation/SpaceWorkflowSaveForm.vue", () => ({ default: { template: "<div />" } }));

import SpaceWorkflowCapture from "../workflows/presentation/SpaceWorkflowCapture.vue";

describe("workflow capture query", () => {
  it("submits the selected node ids rather than the query namespace", async () => {
    const wrapper = shallowMount(SpaceWorkflowCapture);
    const options = captured.options?.value;
    if (!options) throw new Error("Missing capture query");
    await options.queryFn();
    expect(captured.preview).toHaveBeenCalledWith("space-1", ["output-1"]);
    expect(options.queryKey).toEqual(["spaces", "space-1", "workflow-capture-preview", ["output-1"]]);
    wrapper.unmount();
  });
});
