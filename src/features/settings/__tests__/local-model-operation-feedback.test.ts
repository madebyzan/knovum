import { flushPromises } from "@vue/test-utils";
import { expect, it } from "vitest";

import UiProgress from "../../../shared/ui/UiProgress.vue";
import { mocks, render } from "./support/local-model-controls";

it("shows a synchronous preparation startup failure and makes it visible to its capability", async () => {
  mocks.prepare.mockImplementation(() => {
    throw new Error("Unable to start preparation");
  });
  const wrapper = render();
  await wrapper.get('[data-testid="model-download"]').trigger("click");
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe("Unable to start preparation");
  expect(wrapper.emitted("failure")).toHaveLength(1);
  expect(wrapper.emitted("preparationChange")).toBeUndefined();
  expect(wrapper.get('[data-testid="model-download"]').attributes("disabled")).toBeUndefined();
});
it("waits for actual cancellation completion without submitting duplicate requests", async () => {
  let finish!: () => void;
  let acknowledge!: () => void;
  mocks.cancel.mockImplementation(
    () =>
      new Promise<void>((resolve) => {
        acknowledge = resolve;
      }),
  );
  mocks.prepare.mockReturnValue({
    completion: new Promise<void>((resolve) => {
      finish = resolve;
    }),
    cancel: mocks.cancel,
  });
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  await wrapper.get("button").trigger("click");
  expect(wrapper.get("button").attributes("disabled")).toBeDefined();
  await wrapper.get("button").trigger("click");
  expect(mocks.cancel).toHaveBeenCalledOnce();
  acknowledge();
  await flushPromises();
  expect(wrapper.get("button").text()).toBe("settings.ai.localModels.cancelling");
  expect(wrapper.get("button").attributes("disabled")).toBeDefined();
  finish();
  await flushPromises();
  expect(wrapper.find('[data-testid="model-download"]').exists()).toBe(true);
});
it("allows retrying cancellation when the cancellation command fails", async () => {
  let finish!: () => void;
  mocks.prepare.mockReturnValue({
    completion: new Promise<void>((resolve) => {
      finish = resolve;
    }),
    cancel: mocks.cancel,
  });
  mocks.cancel.mockRejectedValueOnce(new Error("Cancellation failed")).mockResolvedValueOnce();
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe("Cancellation failed");
  expect(wrapper.get("button").attributes("disabled")).toBeUndefined();
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(mocks.cancel).toHaveBeenCalledTimes(2);
  expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  finish();
  await flushPromises();
});
it("shows measured transfer progress and removes the percentage during verification", async () => {
  let finish!: () => void;
  mocks.prepare.mockReturnValue({
    completion: new Promise<void>((resolve) => {
      finish = resolve;
    }),
    cancel: mocks.cancel,
  });
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  const onProgress = mocks.prepare.mock.calls[0]![2]!;
  onProgress({ stage: "downloading", downloaded: 25, total: 100, percent: 25 });
  await wrapper.vm.$nextTick();
  expect(wrapper.getComponent(UiProgress).props("value")).toBe(25);
  onProgress({ stage: "verifying", downloaded: 100, total: 100, percent: 100 });
  await wrapper.vm.$nextTick();
  expect(wrapper.findComponent(UiProgress).exists()).toBe(false);
  expect(wrapper.get('[role="status"]').text()).toContain("tasks.resources.verifying");
  finish();
  await flushPromises();
});
it("locks model actions while confirming removal and restores them when dismissed", async () => {
  let dismiss!: (confirmed: boolean) => void;
  mocks.confirm.mockImplementation(
    () =>
      new Promise<boolean>((resolve) => {
        dismiss = resolve;
      }),
  );
  const wrapper = render({ installed: true });
  await wrapper.get('[data-testid="model-remove"]').trigger("click");
  expect(wrapper.get('[data-testid="model-remove"]').attributes("disabled")).toBeDefined();
  expect(wrapper.get('[data-testid="model-download"]').attributes("disabled")).toBeDefined();
  await wrapper.get('[data-testid="model-remove"]').trigger("click");
  expect(mocks.confirm).toHaveBeenCalledOnce();
  dismiss(false);
  await flushPromises();
  expect(mocks.remove).not.toHaveBeenCalled();
  expect(wrapper.get('[data-testid="model-remove"]').attributes("disabled")).toBeUndefined();
});
it("reports confirmation failures without leaving model actions locked", async () => {
  mocks.confirm.mockRejectedValue(new Error("Confirmation unavailable"));
  const wrapper = render({ installed: true });
  await wrapper.get('[data-testid="model-remove"]').trigger("click");
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe("Confirmation unavailable");
  expect(wrapper.get('[data-testid="model-remove"]').attributes("disabled")).toBeUndefined();
});

it("keeps keyboard focus in the model when its action is replaced", async () => {
  let finish!: () => void;
  mocks.prepare.mockReturnValue({
    completion: new Promise<void>((resolve) => {
      finish = resolve;
    }),
    cancel: mocks.cancel,
  });
  const wrapper = render();
  (wrapper.get("button").element as HTMLButtonElement).focus();
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(document.activeElement).toBe(wrapper.get("button").element);
  expect(wrapper.get("button").text()).toBe("common.cancel");
  finish();
  await flushPromises();
  expect(document.activeElement).toBe(wrapper.get('[data-testid="model-download"]').element);
});
it("treats a completed cancellation as a normal outcome", async () => {
  mocks.prepare.mockReturnValue({
    completion: Promise.reject({ code: "local_resource_preparation_cancelled", message: "Cancelled" }),
    cancel: mocks.cancel,
  });
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  expect(wrapper.emitted("failure")).toBeUndefined();
});
