import type { SpaceActionDefinition } from "../../../../foundation/ai/space-action-catalog";

function objectValue(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

export function workflowParameterOptions(action: SpaceActionDefinition | undefined, key: string) {
  const property = objectValue(objectValue(action?.parametersSchema.properties)?.[key]);
  if (!property) return [];
  const reference = property.$ref;
  const definitions = objectValue(action?.parametersSchema.$defs);
  const resolved =
    typeof reference === "string" && reference.startsWith("#/$defs/")
      ? objectValue(definitions?.[reference.slice(8)])
      : property;
  const values = resolved?.enum;
  if (!Array.isArray(values)) return [];
  return values
    .filter((value): value is string | number => typeof value === "string" || typeof value === "number")
    .map((value) => ({ value: String(value), label: String(value) }));
}
