export function createAsyncDisposer() {
  let disposed = false;
  let current: (() => void) | undefined;

  async function attach(registration: Promise<() => void>) {
    const next = await registration;
    if (disposed) {
      next();
      return;
    }
    current?.();
    current = next;
  }

  function dispose() {
    disposed = true;
    current?.();
    current = undefined;
  }

  return { attach, dispose };
}
