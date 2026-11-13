import { shallowRef } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import type { ImageLayersModels } from "../../../contracts/generated/ImageLayersModels";
import {
  readImageLayersModelChoice,
  type ImageLayersChoiceGroup,
} from "../../../knowledge-derivations/image-layers-models";
import { parseImageExecutionTarget } from "../../../knowledge-derivations/image-model-targets";

interface PendingLayersChoice {
  groups: ImageLayersChoiceGroup[];
  resolved: Partial<ImageLayersModels>;
}

/** Layer splitting chooses up to four models at once; the node bar becomes one compact form. */
export function useImageLayersChoice(invoke: (models: ImageLayersModels) => void) {
  const pending = shallowRef<PendingLayersChoice>();

  async function begin(): Promise<void> {
    try {
      const decision = await readImageLayersModelChoice();
      if (decision.kind === "choose") {
        pending.value = { groups: decision.groups, resolved: decision.resolved };
        return;
      }
      invoke(decision.models);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  function run(selections: Record<string, string>) {
    const choice = pending.value;
    if (!choice) return;
    const models = { ...choice.resolved } as ImageLayersModels;
    for (const group of choice.groups) models[group.key] = parseImageExecutionTarget(selections[group.key]!);
    pending.value = undefined;
    invoke(models);
  }

  function cancel() {
    pending.value = undefined;
  }

  return { pending, begin, run, cancel };
}
