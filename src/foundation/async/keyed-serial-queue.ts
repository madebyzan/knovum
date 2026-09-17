export interface KeyedSerialQueue<Key> {
  run<Value>(key: Key, task: () => Promise<Value>): Promise<Value>;
}

export function createKeyedSerialQueue<Key>(): KeyedSerialQueue<Key> {
  const tails = new Map<Key, Promise<void>>();

  return {
    run<Value>(key: Key, task: () => Promise<Value>): Promise<Value> {
      const previous = tails.get(key) ?? Promise.resolve();
      const current = previous.catch(() => undefined).then(task);
      const tail = current.then(
        () => undefined,
        () => undefined,
      );
      tails.set(key, tail);
      void tail.finally(() => {
        if (tails.get(key) === tail) tails.delete(key);
      });
      return current;
    },
  };
}
