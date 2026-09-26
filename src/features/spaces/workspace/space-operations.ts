import { pick } from "es-toolkit";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { Space } from "../model/space";

export function listSpaces(): Promise<Space[]> {
  return invoke("plugin:spaces|list_spaces");
}

export function createSpace(space: Space): Promise<void> {
  return invoke("plugin:spaces|create_space", {
    space: pick(space, ["id", "kind", "title"]),
  });
}

export function deleteSpacePermanently(spaceId: string): Promise<string[]> {
  return invoke<string[]>("plugin:spaces|delete_space_permanently", { request: { spaceId } });
}

export function listCredentialNodeIds(spaceId: string): Promise<string[]> {
  return invoke("plugin:spaces|list_credential_node_ids", { spaceId });
}

export function updateSpaceTitle(spaceId: string, title: string): Promise<void> {
  return invoke("plugin:spaces|update_space_title", { spaceId, title });
}

export function archiveSpace(spaceId: string): Promise<void> {
  return invoke("plugin:spaces|archive_space", { spaceId });
}

export function unarchiveSpace(spaceId: string): Promise<void> {
  return invoke("plugin:spaces|unarchive_space", { spaceId });
}

export function markSpaceOpened(spaceId: string): Promise<void> {
  return invoke("plugin:spaces|mark_space_opened", { spaceId });
}

export function setSpaceFavorite(spaceId: string, favorite: boolean): Promise<void> {
  return invoke("plugin:spaces|set_space_favorite", { spaceId, favorite });
}

export function updateSpaceThumbnail(spaceId: string, thumbnailPath: string): Promise<string[]> {
  return invoke("plugin:spaces|update_space_thumbnail", { spaceId, thumbnailPath });
}
