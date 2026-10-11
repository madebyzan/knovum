import type { LocalModelAvailability } from "../../../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelGroup } from "../../../../shared/contracts/generated/LocalModelGroup";

const GROUP_ORDER: readonly LocalModelGroup[] = ["knowledge", "audio", "images"];

export function buildLocalModelGroups(models: readonly LocalModelAvailability[]) {
  return GROUP_ORDER.flatMap((id) => {
    const groupModels = models.filter((model) => model.group === id);
    if (!groupModels.length) return [];
    const capabilities = [...new Set(groupModels.map((model) => model.capability))].map((capability) => ({
      id: capability,
      models: groupModels.filter((model) => model.capability === capability),
    }));
    return [{ id, capabilities }];
  });
}
