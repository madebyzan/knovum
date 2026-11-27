import { shouldShowAction } from "../../../../shared/contracts/action-availability";
import {
  CREATION_INTENT_DEFINITIONS,
  type CreationIntentDecisions,
} from "../catalog/creation-intent-definitions";

/** Returns the generative creation intents (document, image, video) visible for the active selection. */
export function visibleCreationActions(derivations: CreationIntentDecisions) {
  return CREATION_INTENT_DEFINITIONS.filter(
    ({ group, id }) => group === "create" && shouldShowAction(derivations[id]),
  );
}

export function hasVisibleCreationActions(derivations: CreationIntentDecisions): boolean {
  return visibleCreationActions(derivations).length > 0;
}
