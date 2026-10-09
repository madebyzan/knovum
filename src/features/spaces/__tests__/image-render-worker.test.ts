import { afterEach, expect, it, vi } from "vitest";
import { effectScope } from "vue";

import type { ImageRenderRequest } from "../image-editing/image-render-worker";
import { useImageRenderWorker } from "../image-editing/use-image-render-worker";

it("shares one worker, replaces stale jobs and releases stale bitmaps", async () => {
  const postMessage = vi.fn<Worker["postMessage"]>();
  const terminate = vi.fn<() => void>();
  const handlers = new Map<string, EventListener>();
  const constructor = vi.fn<() => Worker>(function () {
    return {
      postMessage,
      terminate,
      addEventListener: (type: string, handler: EventListener) => handlers.set(type, handler),
    } as unknown as Worker;
  });
  vi.stubGlobal("Worker", constructor);
  const first = effectScope();
  const second = effectScope();
  const rendering = first.run(useImageRenderWorker)!;
  expect(second.run(useImageRenderWorker)).toBe(rendering);
  const owner = {};
  const source = { close: vi.fn<() => void>() } as unknown as ImageBitmap;
  const prepare = vi.fn<() => Promise<ImageRenderRequest>>().mockResolvedValue({
    image: source,
    size: { width: 100, height: 100, pixelRatio: 1 },
    edit: {} as ImageRenderRequest["edit"],
  });
  const old = rendering.render(owner, prepare, false);
  await vi.waitFor(() => expect(postMessage).toHaveBeenCalledTimes(1));
  const next = rendering.render(owner, prepare, true);
  await expect(old).resolves.toBeUndefined();
  const stale = { close: vi.fn<() => void>() };
  handlers.get("message")!({ data: { bitmap: stale } } as MessageEvent);
  expect(stale.close).toHaveBeenCalledOnce();
  await vi.waitFor(() => expect(postMessage).toHaveBeenCalledTimes(2));
  const current = { close: vi.fn<() => void>() };
  handlers.get("message")!({ data: { bitmap: current } } as MessageEvent);
  await expect(next).resolves.toBe(current);
  expect(constructor).toHaveBeenCalledOnce();
  first.stop();
  expect(terminate).not.toHaveBeenCalled();
  second.stop();
  expect(terminate).toHaveBeenCalledOnce();
});
it("closes a prepared bitmap if the worker fails before dispatch", async () => {
  const postMessage = vi.fn<Worker["postMessage"]>();
  const terminate = vi.fn<() => void>();
  const handlers = new Map<string, EventListener>();
  vi.stubGlobal("Worker", function () {
    return {
      postMessage,
      terminate,
      addEventListener: (type: string, handler: EventListener) => handlers.set(type, handler),
    };
  });
  const scope = effectScope();
  const rendering = scope.run(useImageRenderWorker)!;
  let complete!: (request: ImageRenderRequest) => void;
  const prepared = new Promise<ImageRenderRequest>((resolve) => {
    complete = resolve;
  });
  const result = rendering.render({}, () => prepared, true);
  const rejected = result.catch((error: unknown) => error);
  handlers.get("error")!({ message: "Worker stopped" } as ErrorEvent);
  expect(await rejected).toEqual(new Error("Worker stopped"));
  const image = { close: vi.fn<() => void>() } as unknown as ImageBitmap;
  complete({
    image,
    size: { width: 100, height: 100, pixelRatio: 1 },
    edit: {} as ImageRenderRequest["edit"],
  });
  await vi.waitFor(() => expect(image.close).toHaveBeenCalledOnce());
  expect(postMessage).not.toHaveBeenCalled();
  expect(terminate).toHaveBeenCalledOnce();
  scope.stop();
});
afterEach(() => vi.unstubAllGlobals());
