import { flushPromises } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, shallowRef, type EffectScope } from "vue";
import type * as VueI18n from "vue-i18n";

import { useVideoEditProxy } from "../use-video-edit-proxy";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
  logError: vi.fn<(message: string) => Promise<void>>(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("vue-i18n", async (original) => ({
  ...(await original<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

let scope: EffectScope;
beforeEach(() => {
  mocks.invoke.mockReset().mockResolvedValue(undefined);
  mocks.logError.mockReset().mockResolvedValue(undefined);
  scope = effectScope();
});
afterEach(() => scope.stop());

function startProxy(nodeId = shallowRef<string | undefined>("source-node")) {
  const proxy = scope.run(() =>
    useVideoEditProxy(nodeId, shallowRef("source-revision"), shallowRef("space-id"), shallowRef(true)),
  );
  if (!proxy) throw new Error("Proxy must run in its effect scope");
  return proxy;
}

describe("video editing proxy lifecycle", () => {
  it("does not prepare an unavailable source", () => {
    const proxy = startProxy(shallowRef(undefined));
    expect(mocks.invoke).not.toHaveBeenCalled();
    expect(proxy.error.value).toContain("sourceUnavailable");
  });

  it("prepares the exact source revision and does not cancel a completed job", async () => {
    mocks.invoke.mockResolvedValue("/cache/proxy.mp4");
    const proxy = startProxy();
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|prepare_video_edit_proxy", {
      request: {
        id: expect.any(String),
        spaceId: "space-id",
        sourceNodeId: "source-node",
        sourceRevision: "source-revision",
      },
    });
    expect(proxy.path.value).toBe("/cache/proxy.mp4");
    scope.stop();
    expect(mocks.invoke).toHaveBeenCalledTimes(1);
  });

  it("cancels a pending job and ignores its late result when the source changes", async () => {
    let complete: ((path: string) => void) | undefined;
    mocks.invoke.mockImplementationOnce(() => new Promise<string>((resolve) => (complete = resolve)));
    const nodeId = shallowRef<string | undefined>("source-node");
    const proxy = startProxy(nodeId);
    const firstCall = mocks.invoke.mock.calls[0]?.[1] as { request: { id: string } };
    nodeId.value = undefined;
    await nextTick();
    expect(mocks.invoke).toHaveBeenLastCalledWith("plugin:spaces|cancel_video_edit_proxy", {
      jobId: firstCall.request.id,
    });
    complete?.("/cache/stale.mp4");
    await flushPromises();
    expect(proxy.path.value).toBeUndefined();
    expect(proxy.error.value).toContain("sourceUnavailable");
  });

  it("cancels hidden preparation, ignores late output and reuses a completed proxy", async () => {
    const visible = shallowRef(true);
    let complete: ((path: string) => void) | undefined;
    mocks.invoke.mockImplementationOnce(() => new Promise<string>((resolve) => (complete = resolve)));
    const proxy = scope.run(() =>
      useVideoEditProxy(shallowRef("node"), shallowRef("revision"), shallowRef("space"), visible),
    )!;
    visible.value = false;
    await nextTick();
    expect(mocks.invoke).toHaveBeenLastCalledWith(
      "plugin:spaces|cancel_video_edit_proxy",
      expect.any(Object),
    );
    complete?.("/cache/stale.mp4");
    await flushPromises();
    expect(proxy.path.value).toBeUndefined();
    mocks.invoke.mockResolvedValueOnce("/cache/current.mp4");
    visible.value = true;
    await flushPromises();
    expect(proxy.path.value).toBe("/cache/current.mp4");
    const calls = mocks.invoke.mock.calls.length;
    visible.value = false;
    await nextTick();
    visible.value = true;
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledTimes(calls);
  });

  it("shows a command failure and prepares a new job on retry", async () => {
    mocks.invoke.mockRejectedValueOnce({ code: "operation_failed", message: "FFmpeg unavailable" });
    mocks.invoke.mockResolvedValueOnce("/cache/retry.mp4");
    const proxy = startProxy();
    await flushPromises();
    expect(proxy.error.value).toContain("FFmpeg unavailable");
    proxy.retry();
    await flushPromises();
    expect(proxy.error.value).toBeUndefined();
    expect(proxy.path.value).toBe("/cache/retry.mp4");
    expect(mocks.invoke).toHaveBeenCalledTimes(2);
  });

  it.each(["", null, 12])("rejects a malformed proxy path: %j", async (path) => {
    mocks.invoke.mockResolvedValue(path);
    const proxy = startProxy();
    await flushPromises();
    expect(proxy.path.value).toBeUndefined();
    expect(proxy.error.value).toContain("proxyFailed");
  });

  it("reports cleanup cancellation failures instead of silently discarding them", async () => {
    mocks.invoke.mockImplementationOnce(() => new Promise(() => undefined));
    mocks.invoke.mockRejectedValueOnce("Command not permitted");
    startProxy();
    scope.stop();
    await flushPromises();
    expect(mocks.logError).toHaveBeenCalledWith(expect.stringContaining("Command not permitted"));
  });
});
