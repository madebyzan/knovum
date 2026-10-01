import assert from "node:assert/strict";
import test from "node:test";

import { github, readStableChannel, requireDistributionRepository, writeStableChannel } from "../release-github.mjs";

const originalToken = process.env.GH_TOKEN;
test.before(() => { process.env.GH_TOKEN = "test-only-token"; });
test.after(() => {
  if (originalToken === undefined) delete process.env.GH_TOKEN;
  else process.env.GH_TOKEN = originalToken;
});

test("only an actual 404 permits first channel creation", async (context) => {
  context.mock.method(globalThis, "fetch", async () => new Response(null, { status: 404 }));
  assert.equal(await readStableChannel("owner/releases"), null);
});

test("authorization and service failures cannot be mistaken for an absent release", async (context) => {
  const request = context.mock.method(globalThis, "fetch", async () => new Response(null, { status: 403 }));
  await assert.rejects(readStableChannel("owner/releases"), /HTTP 403/);
  request.mock.mockImplementation(async () => new Response(null, { status: 503 }));
  await assert.rejects(github("/repos/owner/releases", { allowMissing: true }), /HTTP 503/);
});

test("channel promotion is a compare-and-swap, not a blind overwrite", async (context) => {
  const request = context.mock.method(globalThis, "fetch", async () => Response.json({}));
  await writeStableChannel("owner/releases", { sha: "previous-blob" }, { version: "1.2.3" });
  const [url, options] = request.mock.calls[0].arguments;
  assert.equal(url, "https://api.github.com/repos/owner/releases/contents/channels/stable.json");
  assert.equal(options.redirect, "error");
  assert.equal(options.method, "PUT");
  assert.equal(JSON.parse(options.body).sha, "previous-blob");
  assert.equal(JSON.parse(options.body).branch, "main");
});

test("anonymous distribution refuses private or uninitialized repositories", async (context) => {
  const request = context.mock.method(globalThis, "fetch", async () => Response.json({ private: true, default_branch: "main" }));
  await assert.rejects(requireDistributionRepository("owner/releases"), /public, initialized/);
  request.mock.mockImplementation(async () => Response.json({ private: false, default_branch: "main" }));
  assert.equal(await requireDistributionRepository("owner/releases"), "owner/releases");
});
