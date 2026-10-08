import type { LocalModelAvailability } from "../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelGroup } from "../../shared/contracts/generated/LocalModelGroup";

const groupOrder: readonly LocalModelGroup[] = ["knowledge", "audio", "images"];

export function localModelGroups(models: readonly LocalModelAvailability[]) {
  return groupOrder.flatMap((id) => {
    const entries = models.filter((model) => model.group === id);
    if (!entries.length) return [];
    const capabilities = [...new Set(entries.map((model) => model.capability))].map((capability) => ({
      id: capability,
      models: entries.filter((model) => model.capability === capability),
    }));
    return [{ id, capabilities }];
  });
}
