import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import { relatedContentResponseSchema } from "../model/related-content";

export async function listRelatedContent(spaceId: string, nodeId: string) {
  const value = await invokeCommand<unknown>("plugin:spaces|list_related_content", { spaceId, nodeId });
  return relatedContentResponseSchema.parse(value);
}
