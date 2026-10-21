import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent } from "vue";

import { useVoiceCandidates } from "../../use-voice-candidates";
import { runningRun, voiceHost } from "../support/voice-candidate-host";

/** Mounts the composable inside a Vue Query host and tears it down with the test. */
function setup() {
  let candidates!: ReturnType<typeof useVoiceCandidates>;
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Harness = defineComponent({
    setup() {
      candidates = useVoiceCandidates();
      return () => null;
    },
  });
  const wrapper = mount(Harness, { global: { plugins: [[VueQueryPlugin, { queryClient }]] } });
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  return { candidates, wrapper };
}

/** Restoration, cancellation and settlement of candidate runs. */
export function lifecycleScenarios() {
  describe("voice candidate lifecycle", () => {
    const running = runningRun("voice-run");

    beforeEach(() => {
      vi.resetAllMocks();
      voiceHost.getActiveVoiceCandidateRun.mockResolvedValue(running);
      voiceHost.listVoiceCandidates.mockResolvedValue([]);
      voiceHost.observeResourceRun.mockReturnValue(vi.fn());
    });

    it("restores the application candidate run independently of a Space", async () => {
      const { candidates } = setup();
      await flushPromises();
      expect(voiceHost.getActiveVoiceCandidateRun).toHaveBeenCalledOnce();
      expect(voiceHost.listVoiceCandidates).toHaveBeenCalledOnce();
      expect(candidates.activeRunId.value).toBe(running.id);
    });

    it("settles normal cancellation without presenting a failed voice proposal", async () => {
      const { candidates } = setup();
      await flushPromises();
      voiceHost.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state: "cancelled" });
      expect(candidates.activeRunId.value).toBeUndefined();
      expect(candidates.busy.value).toBe(false);
      expect(candidates.errors.value).toEqual([]);
      expect(voiceHost.getVoiceCandidate).not.toHaveBeenCalled();
    });

    it.each(["failed", "completed_with_errors"] as const)(
      "retains %s as a visible failure",
      async (state) => {
        const { candidates } = setup();
        await flushPromises();
        voiceHost.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state });
        expect(candidates.busy.value).toBe(false);
        expect(candidates.errors.value).not.toEqual([]);
      },
    );

    it("keeps a cancellation failure attached to the active operation", async () => {
      const { candidates } = setup();
      await flushPromises();
      voiceHost.cancelResourceRun.mockRejectedValue(new Error("Cancellation denied"));
      await candidates.cancel();
      expect(candidates.errors.value).toContain("Cancellation denied");
      expect(candidates.activeRunId.value).toBe(running.id);
    });

    it("submits cancellation once and keeps controls busy until its reply settles", async () => {
      const { candidates } = setup();
      await flushPromises();
      let finish!: () => void;
      voiceHost.cancelResourceRun.mockReturnValue(
        new Promise((resolve) => {
          finish = resolve;
        }),
      );
      const first = candidates.cancel();
      const second = candidates.cancel();
      expect(candidates.isCancelling.value).toBe(true);
      expect(voiceHost.cancelResourceRun).toHaveBeenCalledOnce();
      voiceHost.observeResourceRun.mock.calls[0]?.[2]?.({ ...running, state: "cancelled" });
      expect(candidates.busy.value).toBe(true);
      finish();
      await Promise.all([first, second]);
      expect(candidates.isCancelling.value).toBe(false);
      expect(candidates.busy.value).toBe(false);
    });

    it("detaches its view observer without cancelling the durable run", async () => {
      const stop = vi.fn<() => void>();
      voiceHost.observeResourceRun.mockReturnValue(stop);
      const { wrapper } = setup();
      await flushPromises();
      wrapper.unmount();
      expect(stop).toHaveBeenCalledOnce();
      expect(voiceHost.cancelResourceRun).not.toHaveBeenCalled();
    });
  });
}
