import type { SpaceCredential } from "../model/credential";
import { deleteSpaceCredential, saveSpaceCredential } from "./credential-repository";

export async function commitSpaceCredentialChange(
  nodeId: string,
  credential: SpaceCredential | undefined,
  persistMetadata: () => Promise<void>,
): Promise<void> {
  if (credential) await saveSpaceCredential(nodeId, credential);
  else await deleteSpaceCredential(nodeId);
  await persistMetadata();
}
