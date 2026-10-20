import type { McpElicitationField } from "./schema/mcp-elicitation";

export type McpElicitationValue = string | number | boolean | string[];
export function elicitationOptions(
  field: McpElicitationField,
): { value: string | number; label: string }[] | undefined {
  if (field.type === "array")
    return "enum" in field.items
      ? field.items.enum.map((value) => ({ value, label: value }))
      : field.items.anyOf.map((item) => ({ value: item.const, label: item.title }));
  if (field.type === "string" && field.oneOf)
    return field.oneOf.map((item) => ({ value: item.const, label: item.title }));
  if (field.type !== "boolean" && field.enum)
    return field.enum.map((value, index) => ({
      value,
      label: field.type === "string" ? (field.enumNames?.[index] ?? String(value)) : String(value),
    }));
  return undefined;
}
