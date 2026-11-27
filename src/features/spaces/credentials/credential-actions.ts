import type { SpaceCredential } from "../model/schema/credential";
import { deleteSpaceCredential, loadSpaceCredential, saveSpaceCredential } from "./credential-storage";

export type SpaceCredentialField = "api-key" | "username" | "password";

export async function readSpaceCredentialField(
  nodeId: string,
  field: SpaceCredentialField,
): Promise<string | undefined> {
  const stored = await loadSpaceCredential(nodeId);
  if (!stored) return undefined;
  if (field === "api-key") return stored.kind === "api-key" ? stored.secret : undefined;
  if (stored.kind !== "account-password") return undefined;
  return field === "username" ? stored.username : stored.password;
}

export async function commitSpaceCredentialChange(
  nodeId: string,
  credential: SpaceCredential | undefined,
  persistMetadata: () => Promise<void>,
): Promise<void> {
  if (credential) await saveSpaceCredential(nodeId, credential);
  else await deleteSpaceCredential(nodeId);
  await persistMetadata();
}
