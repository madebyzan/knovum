import { refreshEnrichmentJobs } from "../enrichment/enrichment-jobs-query";
import { updateSpaceTitle } from "./space-operations";
import type { SpaceSessionContext } from "./space-session-context";

export async function renameSpace(context: SpaceSessionContext, spaceId: string, title: string) {
  if (context.draftSpace.value?.id === spaceId) {
    context.draftSpace.value = { ...context.draftSpace.value, title, updatedAt: Date.now() };
    return;
  }
  const space = context.spaces.value.find((candidate) => candidate.id === spaceId);
  if (!space || space.kind === "inbox") return;
  await updateSpaceTitle(spaceId, title);
  space.title = title;
  space.updatedAt = Date.now();
  void refreshEnrichmentJobs();
}
