import { isEqual } from "es-toolkit";

import type { SpaceKnowledgeObjectType } from "../knowledge/model/object-types";
import type { SpaceClipboardPayload } from "./payload-types";

/** Existing custom types are reused only when their schema is identical. */
export function prepareClipboardTypes(
  payload: SpaceClipboardPayload,
  existing: readonly SpaceKnowledgeObjectType[],
) {
  const identities = new Map<string, string>();
  const definitions = payload.objectTypes.map((type) => {
    const target = existing.find(
      (candidate) => candidate.id === type.id || candidate.name.toLowerCase() === type.name.toLowerCase(),
    );
    if (target && (target.system || target.name !== type.name || !isEqual(target.fields, type.fields))) {
      throw new Error(`The target Space already has a different custom type named '${type.name}'`);
    }
    const id = target?.id ?? crypto.randomUUID();
    identities.set(type.id, id);
    return { ...type, id };
  });
  return { identities, definitions };
}
