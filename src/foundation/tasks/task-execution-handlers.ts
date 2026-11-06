export type CancelTask = () => Promise<void> | void;
export type OpenTask = () => Promise<void> | void;
/** Takes the person to where what a waiting task needs can be completed. */
export type ResolveTask = () => Promise<void> | void;
export class TaskExecutionHandlers {
  readonly #cancellationByTaskId = new Map<string, CancelTask>();
  readonly #openerByTaskId = new Map<string, OpenTask>();
  readonly #resolverByTaskId = new Map<string, ResolveTask>();

  delete(taskId: string) {
    this.#cancellationByTaskId.delete(taskId);
    this.#openerByTaskId.delete(taskId);
    this.#resolverByTaskId.delete(taskId);
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

  setResolve(taskId: string, handler?: ResolveTask) {
    if (handler) this.#resolverByTaskId.set(taskId, handler);
    else this.#resolverByTaskId.delete(taskId);
  }

  resolve(taskId: string) {
    return this.#resolverByTaskId.get(taskId)?.();
  }

  clear() {
    this.#cancellationByTaskId.clear();
    this.#openerByTaskId.clear();
    this.#resolverByTaskId.clear();
  }
}
