export type CancelTask = () => Promise<void> | void;
export class TaskExecutionHandlers {
  readonly #cancellations = new Map<string, CancelTask>();

  delete(id: string) {
    this.#cancellations.delete(id);
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

  clear() {
    this.#cancellations.clear();
  }
}
