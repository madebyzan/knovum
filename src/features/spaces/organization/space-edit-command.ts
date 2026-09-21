import type { CanvasNodeSizeAuthority } from "../canvas/contract/canvas-document";
import type { StructureNodeId, StructureLayout } from "../model/structure/structure-contract";

type ContentLayout = Extract<StructureLayout, { kind: "content" }>;
export type FieldUpdate<T> = { kind: "keep" } | { kind: "clear" } | { kind: "set"; value: T };

export interface SpacePoint {
  x: number;
  y: number;
}

export interface SpaceRect extends SpacePoint {
  width: number;
  height: number;
}

/** Product intents accepted by the Rust Space edit transaction. */
export type SpaceEditCommand =
  | {
      kind: "section.create-empty";
      sectionId: StructureNodeId;
      title: string;
      frame: SpaceRect;
      rootOrder: number;
    }
  | {
      kind: "section.draw";
      sectionId: StructureNodeId;
      title: string;
      frame: SpaceRect;
      containedRootIds: StructureNodeId[];
    }
  | { kind: "section.wrap-selection"; sectionId: StructureNodeId; title: string; rootIds: StructureNodeId[] }
  | { kind: "section.resize-frame"; structureNodeId: StructureNodeId; frame: SpaceRect }
  | { kind: "section.fit"; structureNodeId: StructureNodeId }
  | { kind: "section.set-collapsed"; structureNodeId: StructureNodeId; collapsed: boolean }
  | { kind: "section.dissolve"; structureNodeId: StructureNodeId }
  | { kind: "group.create"; groupId: StructureNodeId; rootIds: StructureNodeId[]; title: string }
  | { kind: "group.dissolve"; rootIds: StructureNodeId[] }
  | { kind: "structure.move"; rootIds: StructureNodeId[]; delta: SpacePoint }
  | {
      kind: "structure.move-to-container";
      rootIds: StructureNodeId[];
      delta: SpacePoint;
      parentId: StructureNodeId | null;
      order: number;
      expandSectionId?: StructureNodeId;
    }
  | {
      kind: "structure.transform";
      rootIds: StructureNodeId[];
      anchor: SpacePoint;
      translation: SpacePoint;
      rotationDelta: number;
      scaleFactor: number;
    }
  | {
      kind: "structure.reparent";
      rootIds: StructureNodeId[];
      parentId: StructureNodeId | null;
      order: number;
    }
  | {
      kind: "structure.align";
      rootIds: StructureNodeId[];
      edge: "left" | "center" | "right" | "top" | "middle" | "bottom";
    }
  | { kind: "structure.distribute"; rootIds: StructureNodeId[]; axis: "horizontal" | "vertical" }
  | {
      kind: "structure.reorder";
      rootIds: StructureNodeId[];
      direction: "front" | "forward" | "backward" | "back";
    }
  | { kind: "structure.auto-layout"; rootIds: StructureNodeId[]; direction: "right" | "down"; gap: number }
  | {
      kind: "structure.clone";
      rootIds: StructureNodeId[];
      targetParentId: StructureNodeId | null;
      offset: SpacePoint;
    }
  | { kind: "structure.delete-subtrees"; rootIds: StructureNodeId[] }
  | { kind: "structure.set-lock"; rootIds: StructureNodeId[]; locked: boolean }
  | { kind: "container.rename"; structureNodeId: StructureNodeId; title: string }
  | {
      kind: "content.update-layout";
      structureNodeId: StructureNodeId;
      transform?: ContentLayout["transform"];
      size?: ContentLayout["size"];
      sizeAuthority?: ContentLayout["sizeAuthority"];
      appearance?: FieldUpdate<NonNullable<ContentLayout["appearance"]>>;
      fileSubpath?: FieldUpdate<string>;
    }
  | {
      kind: "content.update-heights";
      updates: Array<{
        structureNodeId: StructureNodeId;
        contentRevision: string | null;
        height: number;
        measuredWidth: number;
        sizeAuthority: CanvasNodeSizeAuthority;
        typographyScale: number;
      }>;
    }
  | {
      kind: "relation.set-style";
      relationId: string;
      color: "blue" | "green" | "amber" | "red" | "purple" | null;
    };

export interface ExecuteSpaceEditRequest {
  spaceId: string;
  mutationId: string;
  expectedEditRevision: number;
  command: SpaceEditCommand;
}

export interface SpaceEditReceipt {
  mutationId: string;
  appliedAtRevision: number;
  selectionSuggestion: StructureNodeId[];
}

export function spaceEditCommandRecordsHistory(command: SpaceEditCommand) {
  return command.kind !== "content.update-heights";
}
