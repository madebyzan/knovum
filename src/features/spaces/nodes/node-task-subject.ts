import { camelCase } from "es-toolkit";

import { i18n } from "../../../foundation/i18n/instance";
import type { TaskReference } from "../../../shared/contracts/task-reference";
import type { SpaceNodeKind } from "../model/nodes/node-payload";

/**
 * Names the node a piece of work belongs to. Work that says which node it is doing something to
 * can be shown on that node and stopped from there, instead of only in the Task Center.
 */
export function nodeTaskSubject(nodeId: string, kind: SpaceNodeKind, title: string): TaskReference {
  return {
    kind: "node",
    id: nodeId,
    label: title,
    typeLabel: i18n.global.t(`spaces.nodeTypes.${camelCase(kind)}`),
  };
}
