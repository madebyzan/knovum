import { createSharedComposable } from "@vueuse/core";
import { onScopeDispose } from "vue";

import type { ImageRenderRequest, ImageRenderResponse } from "./image-render-worker";

interface RenderJob {
  owner: object;
  priority: boolean;
  prepare: () => Promise<ImageRenderRequest>;
  resolve: (bitmap: ImageBitmap | undefined) => void;
  reject: (error: unknown) => void;
  cancelled: boolean;
}

export const useImageRenderWorker = createSharedComposable(() => {
  const worker = new Worker(new URL("./image-render-worker.ts", import.meta.url), { type: "module" });
  const queuedJobByOwner = new Map<object, RenderJob>();
  let running: RenderJob | undefined;
  let stopped = false;
  let failure: Error | undefined;

  function cancel(owner: object) {
    const pending = queuedJobByOwner.get(owner);
    if (pending) {
      pending.resolve(undefined);
      queuedJobByOwner.delete(owner);
    }
    if (running?.owner === owner) {
      running.cancelled = true;
      running.resolve(undefined);
    }
  }
  function dispatch() {
    if (running || stopped || failure || queuedJobByOwner.size === 0) return;
    const jobs = [...queuedJobByOwner.values()];
    const job = jobs.find((candidate) => candidate.priority) ?? jobs[0]!;
    queuedJobByOwner.delete(job.owner);
    running = job;
    void job
      .prepare()
      .then((request) => {
        if (stopped || failure || job.cancelled) {
          request.image.close();
          finish();
          return undefined;
        }
        worker.postMessage(request, [request.image]);
        return undefined;
      })
      .catch((error) => {
        job.reject(error);
        finish();
      });
  }
  function finish() {
    running = undefined;
    dispatch();
  }
  worker.addEventListener("message", (event: MessageEvent<ImageRenderResponse>) => {
    const response = event.data;
    if ("error" in response) running?.reject(new Error(response.error));
    else if (!running || running.cancelled) response.bitmap.close();
    else running.resolve(response.bitmap);
    finish();
  });
  worker.addEventListener("error", (event) => {
    failure = new Error(event.message || "Image preview worker failed");
    running?.reject(failure);
    for (const job of queuedJobByOwner.values()) job.reject(failure);
    queuedJobByOwner.clear();
    running = undefined;
    worker.terminate();
  });
  onScopeDispose(() => {
    stopped = true;
    running?.resolve(undefined);
    for (const job of queuedJobByOwner.values()) job.resolve(undefined);
    queuedJobByOwner.clear();
    worker.terminate();
  });
  return {
    cancel,
    render(owner: object, prepare: RenderJob["prepare"], priority: boolean) {
      cancel(owner);
      if (failure) return Promise.reject<ImageBitmap>(failure);
      return new Promise<ImageBitmap | undefined>((resolve, reject) => {
        queuedJobByOwner.set(owner, { owner, prepare, priority, resolve, reject, cancelled: false });
        dispatch();
      });
    },
  };
});
