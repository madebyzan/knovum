import type { StructureCommand, StructureCommandOf } from "./contract";
import type { StructureOperation, StructureOperationResult } from "./operation";
import { alignItems, distributeItems } from "./arrangement";
import { cloneSubtrees } from "./cloning";
import { dissolveContainer, groupSelection } from "./grouping";
import { deleteSubtrees, renameContainer, setItemsLocked } from "./lifecycle";
import { moveItems, transformGroup } from "./movement";
import { dropItems, reparentItems } from "./reparent";
import { createEmptySection, drawSection, sectionAroundSelection } from "./section-creation";
import { fitSection, resizeSection, setSectionCollapsed } from "./section-editing";

type Kind = StructureCommand["kind"];
type CommandByKind<K extends Kind = Kind> = { [P in K]: StructureCommandOf<P> & { kind: P } }[K];
const handlers: { [K in Kind]: (operation: StructureOperation, command: CommandByKind<K>) => StructureOperationResult } = {
  "create-empty-section": createEmptySection,
  "draw-section": drawSection,
  "section-around-selection": sectionAroundSelection,
  "group-selection": groupSelection,
  "dissolve-container": dissolveContainer,
  "move-items": moveItems,
  "reparent-items": reparentItems,
  "drop-items": dropItems,
  "resize-section": resizeSection,
  "fit-section": fitSection,
  "set-section-collapsed": setSectionCollapsed,
  "rename-container": renameContainer,
  "set-items-locked": setItemsLocked,
  "delete-subtrees": deleteSubtrees,
  "transform-group": transformGroup,
  "align-items": alignItems,
  "distribute-items": distributeItems,
  "clone-subtrees": cloneSubtrees,
};
export function executeStructureCommand<K extends Kind>(operation: StructureOperation, command: CommandByKind<K>): StructureOperationResult {
  return handlers[command.kind](operation, command);
}
