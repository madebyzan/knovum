import pLimit, { type LimitFunction } from "p-limit";

export function createKeyedSerialQueue<Key>() {
  const queueByKey = new Map<Key, LimitFunction>();

  return {
    run<Value>(key: Key, task: () => Promise<Value>): Promise<Value> {
      const queue = queueByKey.get(key) ?? pLimit(1);
      queueByKey.set(key, queue);
      return queue(task).finally(() => {
        if (queue.activeCount === 0 && queue.pendingCount === 0 && queueByKey.get(key) === queue)
          queueByKey.delete(key);
      });
    },
  };
}
