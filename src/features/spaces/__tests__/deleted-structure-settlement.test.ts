import { beforeEach, describe, expect, it, vi } from "vitest";

const bridge = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));
const mutations = vi.hoisted(() => ({
  confirmed: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));
const projections = vi.hoisted(() => ({
  invalidate: vi.fn<(filters?: unknown) => Promise<void>>(),
}));
const logs = vi.hoisted(() => ({ warn: vi.fn<(message: string) => void>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: bridge.invoke }));
vi.mock("@tauri-apps/plugin-log", () => ({ warn: logs.warn }));
vi.mock("../../../foundation/query/query-client", () => ({
  queryClient: { invalidateQueries: projections.invalidate },
}));
vi.mock("../structure/confirmed-mutation", () => ({ invokeConfirmedMutation: mutations.confirmed }));
vi.mock("../structure/space-edit-operations", () => ({
  spaceEditOperations: {
    load: vi.fn<() => Promise<{ editRevision: number }>>().mockResolvedValue({ editRevision: 3 }),
  },
}));
vi.mock("../assets/asset-operations", () => ({
  cleanupDeletedSpaceAssets: vi.fn<(paths: string[]) => Promise<void>>(),
}));

import type { SpaceDeletedStructureRoot } from "../deletion/deleted-structure";
import { permanentlyDeleteStructure, restoreDeletedStructure } from "../deletion/deleted-structure-actions";

const deletedRoot: SpaceDeletedStructureRoot = {
  spaceId: "space",
  batchId: "batch",
  rootStructureNodeId: "root",
  deletedAt: 1,
  title: "Section",
  kind: "section",
  nodeId: null,
  spaceTitle: "Space",
};

beforeEach(() => {
  vi.clearAllMocks();
  mutations.confirmed.mockResolvedValue(undefined);
  bridge.invoke.mockResolvedValue([]);
  projections.invalidate.mockResolvedValue(undefined);
});

describe("deleted structure projection settlement", () => {
  it("keeps a committed restore successful when invalidation fails", async () => {
    projections.invalidate.mockRejectedValue(new Error("query client unavailable"));

    await expect(restoreDeletedStructure(deletedRoot, false)).resolves.toBeUndefined();

    expect(logs.warn).toHaveBeenCalledWith(expect.stringContaining("restore committed"));
  });

  it("preserves deletion and invalidation failures", async () => {
    const primary = new Error("database unavailable");
    const projection = new Error("query client unavailable");
    bridge.invoke.mockRejectedValue(primary);
    projections.invalidate.mockRejectedValue(projection);

    const error = await permanentlyDeleteStructure([deletedRoot], false).catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(AggregateError);
    expect(error).toMatchObject({ cause: primary });
    expect((error as AggregateError).errors).toEqual([primary, projection]);
  });
});
