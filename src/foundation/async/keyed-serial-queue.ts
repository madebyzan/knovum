import pLimit, { type LimitFunction } from "p-limit";

export function createKeyedSerialQueue<Key>() {
  const queues = new Map<Key, LimitFunction>();

  return {
    run<Value>(key: Key, task: () => Promise<Value>): Promise<Value> {
      const queue = queues.get(key) ?? pLimit(1);
      queues.set(key, queue);
      return queue(task).finally(() => {
        if (queue.activeCount === 0 && queue.pendingCount === 0 && queues.get(key) === queue)
          queues.delete(key);
      });
    },
  };
}
