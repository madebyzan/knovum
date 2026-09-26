import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";
import { relatedContentResponseSchema } from "../model/related-content";

export async function listRelatedContent(spaceId: string, nodeId: string) {
  const value = await invoke<unknown>("plugin:spaces|list_related_content", { spaceId, nodeId });
  return relatedContentResponseSchema.parse(value);
}
