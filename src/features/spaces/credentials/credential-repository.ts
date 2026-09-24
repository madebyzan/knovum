import { deleteSecret, getSecret, setSecret } from "../../../foundation/secrets/secret-store";
import { spaceCredentialSchema } from "../model/credential";
import type { SpaceCredential } from "../model/credential";

function secretKey(nodeId: string) {
  if (!/^[\w-]{1,80}$/u.test(nodeId)) throw new Error("Invalid credential node id");
  return `spaces.credential:${nodeId}`;
}

export async function loadSpaceCredential(nodeId: string): Promise<SpaceCredential | undefined> {
  const value = await getSecret(secretKey(nodeId));
  if (!value) return undefined;
  return spaceCredentialSchema.parse(JSON.parse(value));
}

export async function saveSpaceCredential(nodeId: string, credential: SpaceCredential): Promise<void> {
  await setSecret(secretKey(nodeId), JSON.stringify(spaceCredentialSchema.parse(credential)));
}

export async function deleteSpaceCredential(nodeId: string): Promise<void> {
  await deleteSecret(secretKey(nodeId));
}

export async function deleteSpaceCredentials(nodeIds: readonly string[]): Promise<void> {
  for (const nodeId of nodeIds) {
    // Stop at the first deletion failure so credential cleanup cannot be reported as complete.
    // oxlint-disable-next-line no-await-in-loop
    await deleteSpaceCredential(nodeId);
  }
}
