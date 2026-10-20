import { beforeEach, describe, expect, it, vi } from "vitest";

import { createTestSessionContext, createTestSpace } from "../../__tests__/support/session-context";
import type { SpaceCredential } from "../../model/schema/credential";
import type { Space } from "../../model/space";
import { archiveSpace, permanentlyDeleteSpace, unarchiveSpace } from "../archive-actions";
import type { SpaceSessionContext } from "../space-session-context";

const lifecycleRepository = vi.hoisted(() => ({
  archiveSpace: vi.fn<(spaceId: string) => Promise<void>>(),
  unarchiveSpace: vi.fn<(spaceId: string) => Promise<void>>(),
}));
const spaceRepository = vi.hoisted(() => ({
  deleteSpacePermanently: vi.fn<(spaceId: string) => Promise<string[]>>(),
  listCredentialNodeIds: vi.fn<(spaceId: string) => Promise<string[]>>(),
}));

vi.mock("../space-operations", () => ({
  archiveSpace: lifecycleRepository.archiveSpace,
  deleteSpacePermanently: spaceRepository.deleteSpacePermanently,
  listCredentialNodeIds: spaceRepository.listCredentialNodeIds,
  markSpaceOpened: vi.fn<(spaceId: string) => Promise<void>>(),
  unarchiveSpace: lifecycleRepository.unarchiveSpace,
}));
vi.mock("../../session/space-projection", () => ({
  loadSpaceSessionProjection: vi.fn<() => unknown>(() => ({
    nodes: [],
    scenes: [],
    relations: [],
    editSnapshot: { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
  })),
}));
const assets = vi.hoisted(() => ({
  cleanupDeletedSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
}));
vi.mock("../../assets/asset-operations", () => assets);
const credentialStore = vi.hoisted(() => ({
  deleteSpaceCredentials: vi.fn<(nodeIds: readonly string[]) => Promise<void>>(),
  loadSpaceCredential: vi.fn<(nodeId: string) => Promise<SpaceCredential | undefined>>(),
  saveSpaceCredential: vi.fn<(nodeId: string, credential: SpaceCredential) => Promise<void>>(),
}));
vi.mock("../../credentials/credential-storage", () => credentialStore);
function space(): Space {
  return createTestSpace({
    id: "research",
    kind: "space",
    title: "Research",
    lastOpenedAt: 2,
    createdAt: 2,
    updatedAt: 2,
  });
}

function context(research: Space): SpaceSessionContext {
  return createTestSessionContext({ spaces: [research] });
}

describe("Space lifecycle", () => {
  beforeEach(() => vi.clearAllMocks());

  it("archives without destroying the Space", async () => {
    const research = space();
    const state = context(research);
    await archiveSpace(state, research.id);
    expect(research.archivedAt).toEqual(expect.any(Number));
    expect(state.spaces.value.some((candidate: Space) => candidate.id === research.id)).toBe(true);
  });

  it("moves the workbench to the Inbox tab when the active Space is archived", async () => {
    const research = space();
    const state = createTestSessionContext({
      spaces: [createTestSpace(), research],
      activeSpaceId: research.id,
    });

    await archiveSpace(state, research.id);

    expect(state.activeSpaceId.value).toBe("inbox");
    expect(state.tabs.value.map(({ spaceId }) => spaceId)).toEqual(["inbox"]);
    expect(state.closedTabs.value).toHaveLength(0);
  });

  it("restores an archived Space", async () => {
    const research = space();
    const state = context(research);
    await archiveSpace(state, research.id);
    await unarchiveSpace(state, research.id);
    expect(research.archivedAt).toBeNull();
  });

  it("permanently deletes only archived Spaces", async () => {
    const research = space();
    const state = context(research);
    await permanentlyDeleteSpace(state, research.id);
    expect(spaceRepository.deleteSpacePermanently).not.toHaveBeenCalled();
    research.archivedAt = 3;
    spaceRepository.listCredentialNodeIds.mockResolvedValue([]);
    spaceRepository.deleteSpacePermanently.mockResolvedValue(["/asset.png"]);
    await permanentlyDeleteSpace(state, research.id);
    expect(assets.cleanupDeletedSpaceAssets).toHaveBeenCalledWith(["/asset.png"]);
    expect(state.spaces.value).toHaveLength(0);
  });

  it("restores credentials if keychain cleanup partially fails", async () => {
    const research = space();
    research.archivedAt = 3;
    const state = context(research);
    const backup: SpaceCredential = { kind: "api-key", secret: "secret" };
    spaceRepository.listCredentialNodeIds.mockResolvedValue(["credential-node"]);
    credentialStore.loadSpaceCredential.mockResolvedValue(backup);
    credentialStore.deleteSpaceCredentials.mockRejectedValueOnce(new Error("keychain failure"));

    await expect(permanentlyDeleteSpace(state, research.id)).rejects.toThrow("keychain failure");

    expect(credentialStore.saveSpaceCredential).toHaveBeenCalledWith("credential-node", backup);
    expect(spaceRepository.deleteSpacePermanently).not.toHaveBeenCalled();
    expect(state.spaces.value).toHaveLength(1);
  });

  it("restores credentials if permanent database deletion fails", async () => {
    const research = space();
    research.archivedAt = 3;
    const state = context(research);
    const backup: SpaceCredential = { kind: "api-key", secret: "secret" };
    spaceRepository.listCredentialNodeIds.mockResolvedValue(["credential-node"]);
    credentialStore.loadSpaceCredential.mockResolvedValue(backup);
    spaceRepository.deleteSpacePermanently.mockRejectedValueOnce(new Error("database failure"));

    await expect(permanentlyDeleteSpace(state, research.id)).rejects.toThrow("database failure");

    expect(credentialStore.deleteSpaceCredentials).toHaveBeenCalledWith(["credential-node"]);
    expect(credentialStore.saveSpaceCredential).toHaveBeenCalledWith("credential-node", backup);
    expect(state.spaces.value).toHaveLength(1);
  });

  it("preserves database and credential rollback failures", async () => {
    const research = space();
    research.archivedAt = 3;
    const state = context(research);
    const primary = new Error("database failure");
    const rollback = new Error("credential restore failure");
    spaceRepository.listCredentialNodeIds.mockResolvedValue(["credential-node"]);
    credentialStore.loadSpaceCredential.mockResolvedValue({ kind: "api-key", secret: "secret" });
    spaceRepository.deleteSpacePermanently.mockRejectedValueOnce(primary);
    credentialStore.saveSpaceCredential.mockRejectedValueOnce(rollback);

    const error = await permanentlyDeleteSpace(state, research.id).catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(AggregateError);
    expect(error).toMatchObject({ cause: primary });
    expect((error as AggregateError).errors).toEqual([primary, rollback]);
  });
});
