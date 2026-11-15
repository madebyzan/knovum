import assert from "node:assert/strict";
import test from "node:test";

import worker from "../src/index.js";

const payload = {
  installationId: "f1f2f34a-9fc2-4d9c-8b6e-29c137bbfbe4",
  event: "app_started",
  appVersion: "0.3.5",
  platform: "macos",
};

function createDatabase() {
  const calls = [];
  return {
    calls,
    prepare(sql) {
      return {
        bind(...values) {
          return {
            async run() {
              calls.push({ sql, values });
              return { success: true };
            },
          };
        },
      };
    },
  };
}

function createRequest(body = payload) {
  return new Request("https://analytics.example.workers.dev/event", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
}

test("accepts the minimal app-start event and stores only a one-way digest", async () => {
  const DB = createDatabase();
  const response = await worker.fetch(createRequest(), {
    DB,
    INSTALLATION_HASH_SALT: "a".repeat(64),
  });

  assert.equal(response.status, 204);
  assert.equal(DB.calls.length, 1);
  assert.match(DB.calls[0].values[0], /^[0-9a-f]{64}$/);
  assert.notEqual(DB.calls[0].values[0], payload.installationId);
  assert.match(DB.calls[0].sql, /DO NOTHING/u);
  assert.equal(DB.calls[0].values[2], "app_started");
  assert.equal(DB.calls[0].values[3], "0.3.5");
  assert.equal(DB.calls[0].values[4], "macos");
});

test("rejects oversized request bodies before parsing", async () => {
  const DB = createDatabase();
  const request = new Request("https://analytics.example.workers.dev/event", {
    method: "POST",
    body: "x".repeat(1024),
  });
  const response = await worker.fetch(request, {
    DB,
    INSTALLATION_HASH_SALT: "a".repeat(64),
  });

  assert.equal(response.status, 413);
  assert.equal(DB.calls.length, 0);
});

test("rejects extra fields so content cannot be smuggled into the event", async () => {
  const DB = createDatabase();
  const response = await worker.fetch(
    createRequest({ ...payload, note: "private note text" }),
    { DB, INSTALLATION_HASH_SALT: "a".repeat(64) },
  );

  assert.equal(response.status, 400);
  assert.equal(DB.calls.length, 0);
});

test("rejects unsupported event names and platforms", async () => {
  const DB = createDatabase();
  const badEvent = await worker.fetch(
    createRequest({ ...payload, event: "search_performed" }),
    { DB, INSTALLATION_HASH_SALT: "a".repeat(64) },
  );
  const badPlatform = await worker.fetch(
    createRequest({ ...payload, platform: "unknown" }),
    { DB, INSTALLATION_HASH_SALT: "a".repeat(64) },
  );

  assert.equal(badEvent.status, 400);
  assert.equal(badPlatform.status, 400);
  assert.equal(DB.calls.length, 0);
});

test("returns a non-success response when the Worker has no configured D1 or salt", async () => {
  const response = await worker.fetch(createRequest(), {});
  assert.equal(response.status, 503);
});

test("does not expose a method or route other than the event endpoint", async () => {
  const getResponse = await worker.fetch(
    new Request("https://analytics.example.workers.dev/event"),
    { DB: createDatabase(), INSTALLATION_HASH_SALT: "a".repeat(64) },
  );
  const otherRoute = await worker.fetch(
    new Request("https://analytics.example.workers.dev/metrics", { method: "POST" }),
    { DB: createDatabase(), INSTALLATION_HASH_SALT: "a".repeat(64) },
  );

  assert.equal(getResponse.status, 404);
  assert.equal(otherRoute.status, 404);
});

test("prunes records older than the retention window", async () => {
  const DB = createDatabase();
  await worker.scheduled({}, { DB });

  assert.equal(DB.calls.length, 1);
  assert.match(DB.calls[0].sql, /DELETE FROM desktop_daily_active/u);
  assert.match(DB.calls[0].values[0], /^\d{4}-\d{2}-\d{2}$/u);
});
