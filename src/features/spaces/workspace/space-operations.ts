import { pick } from "es-toolkit";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { Space } from "../model/space";

export function listSpaces(): Promise<Space[]> {
  return invokeCommand("plugin:spaces|list_spaces");
}

export function createSpace(space: Space): Promise<void> {
  return invokeCommand("plugin:spaces|create_space", {
    space: pick(space, ["id", "kind", "title"]),
  });
}

export function deleteSpacePermanently(spaceId: string): Promise<string[]> {
  return invokeCommand<string[]>("plugin:spaces|delete_space_permanently", { request: { spaceId } });
}

export function listCredentialNodeIds(spaceId: string): Promise<string[]> {
  return invokeCommand("plugin:spaces|list_credential_node_ids", { spaceId });
}

export function updateSpaceTitle(spaceId: string, title: string): Promise<void> {
  return invokeCommand("plugin:spaces|update_space_title", { spaceId, title });
}

export function archiveSpace(spaceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|archive_space", { spaceId });
}

export function unarchiveSpace(spaceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|unarchive_space", { spaceId });
}

export function markSpaceOpened(spaceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|mark_space_opened", { spaceId });
}

export function favoriteSpace(spaceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|favorite_space", { spaceId });
}

export function unfavoriteSpace(spaceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|unfavorite_space", { spaceId });
}

export function updateSpaceThumbnail(spaceId: string, thumbnailPath: string): Promise<string[]> {
  return invokeCommand("plugin:spaces|update_space_thumbnail", { spaceId, thumbnailPath });
}
