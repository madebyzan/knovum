import { cleanupDeletedSpaceAssets } from "../assets/asset-operations";
import {
  deleteSpaceCredentials,
  loadSpaceCredential,
  saveSpaceCredential,
} from "../credentials/credential-storage";
import { refreshEnrichmentJobs } from "../enrichment/enrichment-jobs-query";
import type { SpaceCredential } from "../model/schema/credential";
import {
  archiveSpace as persistSpaceArchive,
  deleteSpacePermanently,
  listCredentialNodeIds,
  unarchiveSpace as persistSpaceUnarchive,
} from "./space-operations";
import { discardSpaceTab, openSpace } from "./space-session-actions";
import type { SpaceSessionContext } from "./space-session-context";

export async function archiveSpace(context: SpaceSessionContext, spaceId: string) {
  const space = editableSpace(context, spaceId);
  if (!space) return;
  const archivedAt = Date.now();
  await persistSpaceArchive(spaceId);
  Object.assign(space, { archivedAt, updatedAt: archivedAt });
  void refreshEnrichmentJobs();
  await leaveUnavailableSpace(context, spaceId);
  discardSpaceTab(context, spaceId);
}

export async function unarchiveSpace(context: SpaceSessionContext, spaceId: string) {
  const space = editableSpace(context, spaceId);
  if (!space) return;
  await persistSpaceUnarchive(spaceId);
  Object.assign(space, { archivedAt: null, updatedAt: Date.now() });
  void refreshEnrichmentJobs();
}

export async function permanentlyDeleteSpace(context: SpaceSessionContext, spaceId: string) {
  const space = editableSpace(context, spaceId);
  if (!space?.archivedAt) return;
  const credentialIds = await listCredentialNodeIds(spaceId);
  const credentialBackups = await Promise.all(
    credentialIds.map(async (nodeId) => [nodeId, await loadSpaceCredential(nodeId)] as const),
  );
  try {
    await deleteSpaceCredentials(credentialIds);
  } catch (error) {
    return throwAfterCredentialRestore(error, credentialBackups);
  }
  let paths: string[];
  try {
    paths = await deleteSpacePermanently(spaceId);
  } catch (error) {
    return throwAfterCredentialRestore(error, credentialBackups);
  }
  context.spaces.value = context.spaces.value.filter((candidate) => candidate.id !== spaceId);
  void refreshEnrichmentJobs();
  discardSpaceTab(context, spaceId);
  await cleanupDeletedSpaceAssets(paths);
}

function editableSpace(context: SpaceSessionContext, spaceId: string) {
  const space = context.spaces.value.find((candidate) => candidate.id === spaceId);
  return space?.kind === "space" ? space : undefined;
}

async function leaveUnavailableSpace(context: SpaceSessionContext, spaceId: string) {
  if (context.activeSpaceId.value !== spaceId) return;
  const inbox = context.spaces.value.find((candidate) => candidate.kind === "inbox");
  if (inbox) await openSpace(context, inbox.id);
}

type CredentialBackup = readonly [nodeId: string, credential: SpaceCredential | undefined];

async function restoreCredentials(backups: readonly CredentialBackup[]) {
  await Promise.all(
    backups.map(([nodeId, credential]) =>
      credential ? saveSpaceCredential(nodeId, credential) : Promise.resolve(),
    ),
  );
}

async function throwAfterCredentialRestore(
  error: unknown,
  backups: readonly CredentialBackup[],
): Promise<never> {
  try {
    await restoreCredentials(backups);
  } catch (rollbackError) {
    throw new AggregateError(
      [error, rollbackError],
      "Space deletion failed and credentials could not be restored",
      {
        cause: error,
      },
    );
  }
  throw error;
}
