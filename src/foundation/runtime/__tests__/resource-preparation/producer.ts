import { describe, expect, it, vi } from "vitest";

import { useTaskStore } from "../../../tasks/task-store";
import { cancelResourcePreparation, prepareResources } from "../../local-resources";
import { startResourcePreparation } from "../../resource-preparation";
import { useTaskLifetime } from "../support/resource-preparation-host";

/** Cancellation, failure and progress behaviour of a running preparation. */
export function producerScenarios() {
  describe("resource preparation producer lifecycle", () => {
    useTaskLifetime();

    it("waits for the actual result when cancellation races successful completion", async () => {
      let complete!: () => void;
      vi.mocked(prepareResources).mockReturnValueOnce(
        new Promise<void>((resolve) => {
          complete = resolve;
        }),
      );
      const preparation = startResourcePreparation({ kind: "transcription" }, "Transcribe");
      const tasks = useTaskStore();
      const id = tasks.running[0]!.id;

      await tasks.cancel(id);

      expect(cancelResourcePreparation).toHaveBeenCalledWith(id);
      expect(tasks.running[0]).toMatchObject({ id, cancelling: true });
      complete();
      await preparation.completion;
      expect(tasks.tasks[0]).toMatchObject({ id, status: "succeeded" });
    });

    it("recognizes backend cancellation without matching an English message", async () => {
      vi.mocked(prepareResources).mockRejectedValueOnce({
        code: "local_resource_preparation_cancelled",
        message: "准备已取消",
      });
      const preparation = startResourcePreparation({ kind: "textEmbedding" }, "Semantic search");

      await expect(preparation.completion).rejects.toMatchObject({
        cause: { code: "local_resource_preparation_cancelled" },
      });
      expect(useTaskStore().tasks[0]).toMatchObject({ status: "cancelled" });
    });

    it("preserves a real producer failure after cancellation was accepted", async () => {
      let fail!: (error: Error) => void;
      vi.mocked(prepareResources).mockReturnValueOnce(
        new Promise<void>((_resolve, reject) => {
          fail = reject;
        }),
      );
      const preparation = startResourcePreparation({ kind: "transcription" }, "Transcribe");
      const tasks = useTaskStore();
      const id = tasks.running[0]!.id;

      await tasks.cancel(id);
      const failure = new Error("Disk write failed after cancellation admission");
      const completion = preparation.completion.catch((cause: unknown) => cause);
      fail(failure);

      expect(await completion).toBe(failure);
      expect(tasks.tasks[0]).toMatchObject({ id, status: "failed", statusMessage: failure.message });
    });

    it("keeps the combined progress when one resource becomes ready before the complete preparation", async () => {
      vi.mocked(prepareResources).mockImplementationOnce(async ({ onProgress }) => {
        onProgress({ stage: "ready", downloaded: 100, total: 400, percent: 25 });
        expect(useTaskStore().tasks[0]).toMatchObject({ status: "running", progress: 25 });
        onProgress({ stage: "downloading", downloaded: 200, total: 400, percent: 50 });
        expect(useTaskStore().tasks[0]).toMatchObject({ status: "running", progress: 50 });
      });

      await startResourcePreparation({ kind: "transcription" }, "Transcribe").completion;
      expect(useTaskStore().tasks[0]).toMatchObject({ status: "succeeded", progress: 100 });
    });

    it("preserves a real preparation failure after cancellation was rejected", async () => {
      let fail!: (error: Error) => void;
      vi.mocked(prepareResources).mockReturnValueOnce(
        new Promise<void>((_resolve, reject) => {
          fail = reject;
        }),
      );
      vi.mocked(cancelResourcePreparation).mockRejectedValueOnce(new Error("Cancellation rejected"));
      const preparation = startResourcePreparation({ kind: "transcription" }, "Transcribe");
      await expect(preparation.cancel()).rejects.toThrow("Cancellation rejected");
      const failure = new Error("Disk write failed");
      const failureSettlement = preparation.completion.catch((error: unknown) => error);
      fail(failure);
      expect(await failureSettlement).toBe(failure);
      expect(useTaskStore().tasks[0]?.status).toBe("failed");
    });
  });
}
