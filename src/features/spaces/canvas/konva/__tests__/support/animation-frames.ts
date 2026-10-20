import { vi } from "vitest";

/** Drives code that paints on animation frames without waiting for a real frame. */
export function useTestAnimationFrames() {
  const pending = new Map<number, FrameRequestCallback>();
  let nextHandle = 1;
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    const handle = nextHandle++;
    pending.set(handle, callback);
    return handle;
  });
  vi.stubGlobal("cancelAnimationFrame", (handle: number) => void pending.delete(handle));
  return {
    pending,
    run() {
      const callbacks = [...pending.entries()];
      pending.clear();
      for (const [handle, callback] of callbacks) callback(handle);
    },
  };
}
