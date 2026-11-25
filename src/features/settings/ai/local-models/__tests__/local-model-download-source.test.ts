import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { LocalModelAvailability } from "../../../../../shared/contracts/generated/LocalModelAvailability";
import UiSelect from "../../../../../shared/ui/UiSelect.vue";
import LocalModelsSettings from "../components/LocalModelsSettings.vue";

const mocks = vi.hoisted(() => ({
  getSource: vi.fn<() => Promise<"huggingface" | "hf-mirror">>(async () => "huggingface"),
  setSource: vi.fn<(source: "huggingface" | "hf-mirror") => Promise<void>>(async () => {}),
}));

vi.mock("../../../../../foundation/runtime/local-resources", async (importOriginal) => ({
  ...(await importOriginal()),
  getModelDownloadSource: mocks.getSource,
  setModelDownloadSource: mocks.setSource,
}));
vi.mock("../../../../../foundation/runtime/use-local-models-query", () => ({
  useLocalModelsQuery: () => ({
    data: shallowRef<LocalModelAvailability[]>([]),
    error: shallowRef<Error>(),
    isPending: shallowRef(false),
    isFetching: shallowRef(false),
    refetch: vi.fn<() => Promise<void>>(),
  }),
}));

beforeEach(() => {
  mocks.getSource.mockReset().mockResolvedValue("huggingface");
  mocks.setSource.mockReset().mockResolvedValue(undefined);
});

it("loads and persists the selected Hugging Face download mirror", async () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(LocalModelsSettings, {
    global: { plugins: [i18n, [VueQueryPlugin, { queryClient }]] },
  });
  onTestFinished(() => wrapper.unmount());

  await flushPromises();
  expect(mocks.getSource).toHaveBeenCalledOnce();

  const mirrorSelect = wrapper
    .findAllComponents(UiSelect)
    .find((select) => select.find('[data-testid="model-download-source"]').exists())!;
  mirrorSelect.vm.$emit("update:modelValue", "hf-mirror");
  await flushPromises();

  expect(mocks.setSource).toHaveBeenCalledWith("hf-mirror");
});
