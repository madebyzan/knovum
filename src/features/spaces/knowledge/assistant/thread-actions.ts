import { ask as askDialog } from "@tauri-apps/plugin-dialog";

import type { GroundedThread } from "../model/grounded-thread";
import { deleteGroundedThread, listGroundedThreads, updateGroundedThreadTitle } from "./grounded-repository";

export async function renameKnowledgeThread(
  id: string,
  spaceId: string,
  title: string,
): Promise<GroundedThread[]> {
  await updateGroundedThreadTitle(id, title);
  return listGroundedThreads(spaceId);
}

export async function deleteKnowledgeThread(
  id: string,
  spaceId: string,
  dialog: { title: string; description: string },
): Promise<GroundedThread[] | undefined> {
  const confirmed = await askDialog(dialog.description, { title: dialog.title, kind: "warning" });
  if (!confirmed) return undefined;
  await deleteGroundedThread(id);
  return listGroundedThreads(spaceId);
}
