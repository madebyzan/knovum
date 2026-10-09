export type CancelTask = () => Promise<void> | void;
export type OpenTask = () => Promise<void> | void;
export class TaskExecutionHandlers {
  readonly #cancellationByTaskId = new Map<string, CancelTask>();
  readonly #openerByTaskId = new Map<string, OpenTask>();

  delete(taskId: string) {
    this.#cancellationByTaskId.delete(taskId);
    this.#openerByTaskId.delete(taskId);
  }

  setCancellation(taskId: string, handler?: CancelTask) {
    if (handler) this.#cancellationByTaskId.set(taskId, handler);
    else this.#cancellationByTaskId.delete(taskId);
  }

  cancellation(taskId: string) {
    return this.#cancellationByTaskId.get(taskId);
  }

  deleteCancellation(taskId: string) {
    this.#cancellationByTaskId.delete(taskId);
  }

  setOpen(taskId: string, handler?: OpenTask) {
    if (handler) this.#openerByTaskId.set(taskId, handler);
    else this.#openerByTaskId.delete(taskId);
  }

  open(taskId: string) {
    return this.#openerByTaskId.get(taskId)?.();
  }

  clear() {
    this.#cancellationByTaskId.clear();
    this.#openerByTaskId.clear();
  }
}
