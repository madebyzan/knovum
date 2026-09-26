import { updateSpaceTitle } from "./space-operations";
import type { SpaceSessionContext } from "./space-session-context";

export async function renameSpace(context: SpaceSessionContext, id: string, title: string) {
  if (context.draftSpace.value?.id === id) {
    context.draftSpace.value = { ...context.draftSpace.value, title, updatedAt: Date.now() };
    return;
  }
  const space = context.spaces.value.find((candidate) => candidate.id === id);
  if (!space || space.kind === "inbox") return;
  await updateSpaceTitle(id, title);
  space.title = title;
  space.updatedAt = Date.now();
}
