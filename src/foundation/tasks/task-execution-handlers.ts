export type CancelTask = () => Promise<void> | void;
export type OpenTask = () => Promise<void> | void;
export class TaskExecutionHandlers {
  readonly #cancellations = new Map<string, CancelTask>();
  readonly #openers = new Map<string, OpenTask>();

  delete(id: string) {
    this.#cancellations.delete(id);
    this.#openers.delete(id);
  }

  setCancellation(id: string, handler?: CancelTask) {
    if (handler) this.#cancellations.set(id, handler);
    else this.#cancellations.delete(id);
  }

  cancellation(id: string) {
    return this.#cancellations.get(id);
  }

  deleteCancellation(id: string) {
    this.#cancellations.delete(id);
  }

  setOpen(id: string, handler?: OpenTask) {
    if (handler) this.#openers.set(id, handler);
    else this.#openers.delete(id);
  }

  open(id: string) {
    return this.#openers.get(id)?.();
  }

  clear() {
    this.#cancellations.clear();
    this.#openers.clear();
  }
}
