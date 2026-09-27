import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";

import type { SpaceCredential } from "../model/schema/credential";
import type { Space } from "../model/space";
import { createTestSessionContext, createTestSpace } from "../test-support/session-context";
import { archiveSpace, permanentlyDeleteSpace, unarchiveSpace } from "../workspace/archive-actions";
import type { SpaceSessionContext } from "../workspace/space-session-actions";

const lifecycleRepository = vi.hoisted(() => ({
  archiveSpace: vi.fn<(id: string) => Promise<void>>(),
  unarchiveSpace: vi.fn<(id: string) => Promise<void>>(),
}));
const spaceRepository = vi.hoisted(() => ({
  deleteSpacePermanently: vi.fn<(id: string) => Promise<string[]>>(),
  listCredentialNodeIds: vi.fn<(id: string) => Promise<string[]>>(),
}));

vi.mock("../workspace/space-operations", () => ({
  archiveSpace: lifecycleRepository.archiveSpace,
  deleteSpacePermanently: spaceRepository.deleteSpacePermanently,
  listCredentialNodeIds: spaceRepository.listCredentialNodeIds,
  unarchiveSpace: lifecycleRepository.unarchiveSpace,
}));
const assets = vi.hoisted(() => ({
  cleanupDeletedSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
}));
vi.mock("../assets/asset-operations", () => assets);
const credentialStore = vi.hoisted(() => ({
  deleteSpaceCredentials: vi.fn<(nodeIds: readonly string[]) => Promise<void>>(),
  loadSpaceCredential: vi.fn<(nodeId: string) => Promise<SpaceCredential | undefined>>(),
  saveSpaceCredential: vi.fn<(nodeId: string, credential: SpaceCredential) => Promise<void>>(),
}));
vi.mock("../credentials/credential-storage", () => credentialStore);
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
  return createTestSessionContext({ spaces: ref([research]) });
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
});
