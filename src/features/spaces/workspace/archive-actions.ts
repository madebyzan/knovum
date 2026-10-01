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

export async function archiveSpace(context: SpaceSessionContext, id: string) {
  const space = editableSpace(context, id);
  if (!space) return;
  const archivedAt = Date.now();
  await persistSpaceArchive(id);
  Object.assign(space, { archivedAt, updatedAt: archivedAt });
  void refreshEnrichmentJobs();
  await leaveUnavailableSpace(context, id);
  discardSpaceTab(context, id);
}

export async function unarchiveSpace(context: SpaceSessionContext, id: string) {
  const space = editableSpace(context, id);
  if (!space) return;
  await persistSpaceUnarchive(id);
  Object.assign(space, { archivedAt: null, updatedAt: Date.now() });
  void refreshEnrichmentJobs();
}

export async function permanentlyDeleteSpace(context: SpaceSessionContext, id: string) {
  const space = editableSpace(context, id);
  if (!space?.archivedAt) return;
  const credentialIds = await listCredentialNodeIds(id);
  const credentialBackups = await Promise.all(
    credentialIds.map(async (nodeId) => [nodeId, await loadSpaceCredential(nodeId)] as const),
  );
  try {
    await deleteSpaceCredentials(credentialIds);
  } catch (error) {
    await restoreCredentials(credentialBackups);
    throw error;
  }
  let paths: string[];
  try {
    paths = await deleteSpacePermanently(id);
  } catch (error) {
    await restoreCredentials(credentialBackups);
    throw error;
  }
  context.spaces.value = context.spaces.value.filter((candidate) => candidate.id !== id);
  void refreshEnrichmentJobs();
  discardSpaceTab(context, id);
  await cleanupDeletedSpaceAssets(paths);
}

function editableSpace(context: SpaceSessionContext, id: string) {
  const space = context.spaces.value.find((candidate) => candidate.id === id);
  return space?.kind === "space" ? space : undefined;
}

async function leaveUnavailableSpace(context: SpaceSessionContext, id: string) {
  if (context.activeSpaceId.value !== id) return;
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
