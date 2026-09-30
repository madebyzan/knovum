import { beforeEach, describe, expect, it, vi } from "vitest";

import { loadGroundedDocumentMetadata } from "../knowledge-derivations/document/metadata";

const ipc = vi.hoisted(() => ({ invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: ipc.invoke }));

const metadata = {
  actionKind: "draft-document",
  executionSource: { type: "model", provider: "openai", model: "model" },
  createdAt: 1,
  sources: [],
  citations: [],
};

describe("grounded document metadata boundary", () => {
  beforeEach(() => vi.clearAllMocks());
  it("preserves absent metadata", async () => {
    ipc.invoke.mockResolvedValue(null);
    await expect(loadGroundedDocumentMetadata("node")).resolves.toBeNull();
  });
  it("preserves valid generated metadata and citation projections", async () => {
    ipc.invoke.mockResolvedValue(metadata);
    await expect(loadGroundedDocumentMetadata("node")).resolves.toEqual(metadata);
  });
  it("rejects malformed stored citations", async () => {
    ipc.invoke.mockResolvedValue({ ...metadata, citations: [{ state: "current", index: 1 }] });
    await expect(loadGroundedDocumentMetadata("node")).rejects.toThrow(/chunkId/u);
  });
});
