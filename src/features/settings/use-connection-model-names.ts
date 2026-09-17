import { computed, toValue, type ComputedRef, type MaybeRefOrGetter } from "vue";

import type { AiConnectionSettings } from "../../foundation/ai/ai-models";

export function useConnectionModelNames(
  connections: MaybeRefOrGetter<readonly AiConnectionSettings[]>,
): ComputedRef<Record<string, string>> {
  return computed(() =>
    Object.fromEntries(
      toValue(connections).flatMap((connection) =>
        connection.models.map((model) => [
          `${connection.id}:${model.id}`,
          `${model.name} · ${connection.name}`,
        ]),
      ),
    ),
  );
}
