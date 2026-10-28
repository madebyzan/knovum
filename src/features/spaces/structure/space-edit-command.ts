import type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";

export function spaceEditCommandRecordsHistory(command: SpaceEditCommand) {
  return command.kind !== "content.update-heights";
}

export type SpaceEditProjection = "structure" | "session";

const PROJECTION_BY_COMMAND_KIND: Record<SpaceEditCommand["kind"], SpaceEditProjection> = {
  "section.create-empty": "structure",
  "section.draw": "structure",
  "section.wrap-selection": "structure",
  "section.resize-frame": "structure",
  "section.fit": "structure",
  "section.set-collapsed": "structure",
  "section.dissolve": "structure",
  "section.update-layout": "structure",
  "group.create": "structure",
  "group.dissolve": "structure",
  "structure.move": "structure",
  "structure.move-to-container": "structure",
  "structure.transform": "structure",
  "structure.reparent": "structure",
  "structure.align": "structure",
  "structure.distribute": "structure",
  "structure.reorder": "structure",
  "structure.auto-layout": "structure",
  "structure.clone": "session",
  "structure.delete-subtrees": "session",
  "structure.set-lock": "structure",
  "container.rename": "structure",
  "content.update-layout": "structure",
  "content.update-heights": "structure",
  "relation.set-style": "structure",
};

export function spaceEditCommandProjection(command: SpaceEditCommand): SpaceEditProjection {
  return PROJECTION_BY_COMMAND_KIND[command.kind];
}
