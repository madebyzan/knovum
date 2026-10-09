import { flushPromises } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { useConversationSpaceRestoration } from "../use-conversation-space-restoration";

const scopes: ReturnType<typeof effectScope>[] = [];
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.stop();
});

function restoreSession(restoreDraft: (spaceId?: string) => Promise<boolean>) {
  const spaceId = shallowRef<string | undefined>("first-space");
  const refresh = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
  const restoreTask = vi.fn<(spaceId: string) => Promise<void>>().mockResolvedValue(undefined);
  const scope = effectScope();
  scopes.push(scope);
  const session = scope.run(() =>
    useConversationSpaceRestoration({
      spaceId,
      reset: vi.fn<() => void>(),
      restoreDraft,
      refresh,
      restoreTask,
    }),
  )!;
  return { session, spaceId, refresh, restoreTask, scope };
}

it("exposes its error and clears it when restoring another space", async () => {
  const restoreDraft = vi
    .fn<(spaceId?: string) => Promise<boolean>>()
    .mockRejectedValueOnce(new Error("Draft unavailable"))
    .mockResolvedValue(true);
  const { session, spaceId, restoreTask } = restoreSession(restoreDraft);
  await flushPromises();
  expect(session.error.value).toBe("Draft unavailable");
  spaceId.value = "second-space";
  await flushPromises();
  expect(session.error.value).toBeUndefined();
  expect(restoreTask).toHaveBeenCalledExactlyOnceWith("second-space");
});

it("ignores a failed restoration from a space that is no longer active", async () => {
  let rejectFirst!: (cause: unknown) => void;
  const restoreDraft = vi
    .fn<(spaceId?: string) => Promise<boolean>>()
    .mockReturnValueOnce(
      new Promise<boolean>((_resolve, reject) => {
        rejectFirst = reject;
      }),
    )
    .mockResolvedValue(true);
  const { session, spaceId, refresh } = restoreSession(restoreDraft);
  spaceId.value = "second-space";
  await flushPromises();
  rejectFirst(new Error("Old space failed"));
  await flushPromises();
  expect(session.error.value).toBeUndefined();
  expect(refresh).toHaveBeenCalledOnce();
});

it("does not refresh after its owning scope is disposed", async () => {
  let finish!: (restored: boolean) => void;
  const { scope, refresh, restoreTask } = restoreSession(
    () =>
      new Promise<boolean>((resolve) => {
        finish = resolve;
      }),
  );
  scope.stop();
  finish(true);
  await flushPromises();
  expect(refresh).not.toHaveBeenCalled();
  expect(restoreTask).not.toHaveBeenCalled();
});
