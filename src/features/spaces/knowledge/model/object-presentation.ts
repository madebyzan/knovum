import { isString } from "es-toolkit";

import type { SpaceJsonValue } from "../../model/json-value";
import { SYSTEM_OBJECT_TYPES, systemKnowledgeObjectKind } from "./object-types";

export function knowledgeObjectTypeKey(objectTypeId: string) {
  return systemKnowledgeObjectKind(objectTypeId);
}

export function knowledgeObjectPreview(
  objectTypeId: string,
  summary: string | undefined,
  properties: Readonly<Record<string, SpaceJsonValue>>,
) {
  const kind = systemKnowledgeObjectKind(objectTypeId);
  const primary = properties[kind ? SYSTEM_OBJECT_TYPES[kind].primaryField : ""];
  const source =
    (isMeaningfulText(primary) && primary) ||
    summary ||
    Object.values(properties).find(isMeaningfulText) ||
    "";
  return markdownPreview(source);
}

function isMeaningfulText(value: SpaceJsonValue | undefined): value is string {
  return isString(value) && value.trim().length > 0;
}

function markdownPreview(value: string) {
  return value
    .replace(/```[^\n]*\n?/gu, "")
    .replace(/`([^`]+)`/gu, "$1")
    .replace(/!\[([^\]]*)\]\([^)]*\)/gu, "$1")
    .replace(/\[([^\]]+)\]\([^)]*\)/gu, "$1")
    .replace(/^\s{0,3}(?:#{1,6}|>|[-+*]|\d+[.)])\s+/gmu, "")
    .replace(/[*_~]+/gu, "")
    .replace(/\s+/gu, " ")
    .trim();
}
