import { distributionRepository, requireValue } from "./release-policy.mjs";

export async function github(path, { method = "GET", body, allowMissing = false } = {}) {
  const response = await fetch(`https://api.github.com${path}`, {
    method,
    redirect: "error",
    signal: AbortSignal.timeout(30_000),
    headers: {
      Authorization: `Bearer ${requireValue(process.env.GH_TOKEN, "GH_TOKEN")}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      "Content-Type": "application/json",
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (allowMissing && response.status === 404) return null;
  if (!response.ok) throw new Error(`GitHub ${method} ${path} failed: HTTP ${response.status}`);
  return response.json();
}

export async function requireDistributionRepository(value) {
  const repository = distributionRepository(value);
  const metadata = await github(`/repos/${repository}`);
  if (metadata.private || metadata.default_branch !== "main") {
    throw new Error("Distribution requires a public, initialized repository with default branch main");
  }
  return repository;
}

export async function readStableChannel(repository) {
  return github(`/repos/${repository}/contents/channels/stable.json?ref=main`, { allowMissing: true });
}

export async function writeStableChannel(repository, previous, manifest) {
  const content = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`).toString("base64");
  await github(`/repos/${repository}/contents/channels/stable.json`, {
    method: "PUT",
    body: {
      message: `release: promote Knovum ${manifest.version}`,
      branch: "main",
      content,
      ...(previous ? { sha: previous.sha } : {}),
    },
  });
}
