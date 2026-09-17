export interface Space {
  id: string;
  kind: "inbox" | "space";
  title: string;
  archivedAt: number | null;
  favorite: boolean;
  thumbnailPath: string | null;
  nodeCount: number;
  lastOpenedAt: number;
  createdAt: number;
  updatedAt: number;
}

export type SpaceLibraryScope = "all" | "recent" | "favorites";

export interface SpaceTargetOption {
  value: string;
  label: string;
}

export function spaceTargetOptions(spaces: readonly Space[], inboxLabel: string): SpaceTargetOption[] {
  return spaces
    .filter((space) => !space.archivedAt)
    .map((space) => ({
      value: space.id,
      label: space.kind === "inbox" ? inboxLabel : space.title,
    }));
}
