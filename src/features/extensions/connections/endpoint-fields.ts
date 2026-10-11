import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";

export interface EndpointField {
  name: string;
  value: string;
}

export function buildEndpointFields(endpoint: McpEndpoint): EndpointField[] {
  return Object.entries(endpoint.type === "stdio" ? endpoint.environment : endpoint.headers).map(
    ([name, value]) => ({ name, value }),
  );
}

export function validateEndpointFields(
  fields: EndpointField[],
  transport: McpEndpoint["type"],
): { fields: Record<string, string> } | { error: string } {
  const names = new Set<string>();
  for (const field of fields) {
    if (!field.name.trim() || field.name !== field.name.trim())
      return { error: "extensions.endpoint.configuration.invalidName" };
    const name = transport === "streamable_http" ? field.name.toLowerCase() : field.name;
    if (names.has(name)) return { error: "extensions.endpoint.configuration.duplicateName" };
    names.add(name);
  }
  return { fields: Object.fromEntries(fields.map(({ name, value }) => [name, value])) };
}
