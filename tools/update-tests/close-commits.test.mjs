import assert from "node:assert/strict";
import { test } from "node:test";

import { CloseCommitCoordinator } from "../../src/foundation/lifecycle/close-commits.ts";

function deferred() {
  let complete;
  const promise = new Promise((resolve) => {
    complete = resolve;
  });
  return { promise, resolve: complete };
}

test("installation follows all registered saves", async () => {
  const coordinator = new CloseCommitCoordinator();
  const calls = [];
  coordinator.register(async () => {
    calls.push("save-a");
  });
  coordinator.register(async () => {
    calls.push("save-b");
  });
  await coordinator.restart(async () => {
    calls.push("install");
  });
  assert.deepEqual(calls, ["save-a", "save-b", "install"]);
  assert.equal(coordinator.restarting, false);
});

test("a failed save prevents installation", async () => {
  const coordinator = new CloseCommitCoordinator();
  coordinator.register(async () => {
    throw new Error("disk full");
  });
  let installed = false;
  await assert.rejects(
    coordinator.restart(async () => {
      installed = true;
    }),
    /disk full/,
  );
  assert.equal(installed, false);
  assert.equal(coordinator.restarting, false);
});

test("failure waits for other saves instead of returning with writes still running", async () => {
  const coordinator = new CloseCommitCoordinator();
  const slow = deferred();
  coordinator.register(async () => {
    throw new Error("failed");
  });
  coordinator.register(() => slow.promise);
  let settled = false;
  const result = coordinator
    .restart(async () => assert.fail("must not install"))
    .catch(() => {
      settled = true;
    });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(settled, false);
  assert.equal(coordinator.restarting, true);
  slow.resolve();
  await result;
  assert.equal(settled, true);
});

test("synchronous callback failure does not skip the remaining saves", async () => {
  const coordinator = new CloseCommitCoordinator();
  let saved = false;
  coordinator.register(() => {
    throw new Error("sync failure");
  });
  coordinator.register(async () => {
    saved = true;
  });
  await assert.rejects(coordinator.flush(), /sync failure/);
  assert.equal(saved, true);
});

test("concurrent close requests share one in-flight save barrier", async () => {
  const coordinator = new CloseCommitCoordinator();
  const save = deferred();
  let count = 0;
  coordinator.register(async () => {
    count += 1;
    await save.promise;
  });
  const first = coordinator.flush();
  const second = coordinator.flush();
  assert.equal(first, second);
  save.resolve();
  await Promise.all([first, second]);
  assert.equal(count, 1);
});

test("a second installation is rejected while the first is saving", async () => {
  const coordinator = new CloseCommitCoordinator();
  const save = deferred();
  coordinator.register(() => save.promise);
  const first = coordinator.restart(async () => {});
  await assert.rejects(
    coordinator.restart(async () => assert.fail("second install")),
    /already in progress/,
  );
  save.resolve();
  await first;
});

test("a second installation is rejected while the installer is running", async () => {
  const coordinator = new CloseCommitCoordinator();
  const installer = deferred();
  const first = coordinator.restart(() => installer.promise);
  await new Promise((resolve) => setImmediate(resolve));
  await assert.rejects(
    coordinator.restart(async () => {}),
    /already in progress/,
  );
  installer.resolve();
  await first;
});

test("failed installation releases the restart guard", async () => {
  const coordinator = new CloseCommitCoordinator();
  await assert.rejects(
    coordinator.restart(async () => {
      throw new Error("permission denied");
    }),
    /permission denied/,
  );
  assert.equal(coordinator.restarting, false);
  await coordinator.restart(async () => {});
});

test("unregistered editors are not saved", async () => {
  const coordinator = new CloseCommitCoordinator();
  const remove = coordinator.register(async () => assert.fail("unregistered callback"));
  remove();
  await coordinator.flush();
});

test("a failed flush can be explicitly retried without caching its rejection", async () => {
  const coordinator = new CloseCommitCoordinator();
  let fail = true;
  coordinator.register(async () => {
    if (fail) throw new Error("save failed");
  });
  await assert.rejects(coordinator.flush());
  fail = false;
  await coordinator.flush();
});
