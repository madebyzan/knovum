import { expect, it, vi } from "vitest";

import { useAiEditorMutation } from "../use-ai-editor-mutation";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

it("owns a single admitted write until it settles", async () => {
  let finish!: (value: boolean) => void;
  const operation = vi.fn<() => Promise<boolean>>(
    () =>
      new Promise<boolean>((resolve) => {
        finish = resolve;
      }),
  );
  const state = useAiEditorMutation();
  const first = state.run(operation);
  const competing = vi.fn<() => Promise<boolean>>(async () => true);
  expect(state.run(competing)).toBe(first);
  expect(state.pending()).toBe(first);
  expect(state.busy.value).toBe(true);
  await Promise.resolve();
  finish(true);
  expect(await first).toBe(true);
  expect(operation).toHaveBeenCalledOnce();
  expect(competing).not.toHaveBeenCalled();
  expect(state.busy.value).toBe(false);
  expect(state.pending()).toBeUndefined();
});

it("a synchronous throw is a failed write, not a stuck busy editor", async () => {
  const state = useAiEditorMutation();
  expect(
    await state.run(() => {
      throw new Error("write failed");
    }),
  ).toBe(false);
  expect(state.error.value).toContain("write failed");
  expect(state.busy.value).toBe(false);
  expect(state.pending()).toBeUndefined();
  expect(await state.run(async () => true)).toBe(true);
  expect(state.error.value).toBe("");
});

it("a credential failure result remains a failure with its specific error", async () => {
  const state = useAiEditorMutation();
  expect(
    await state.run(async () => {
      state.error.value = "credential failure";
      return false;
    }),
  ).toBe(false);
  expect(state.error.value).toBe("credential failure");
  expect(state.pending()).toBeUndefined();
});
