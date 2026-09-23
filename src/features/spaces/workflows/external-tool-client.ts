import { invoke } from "@tauri-apps/api/core";

import type { ExternalToolBinding } from "../contracts/generated/ExternalToolBinding";
import type { SaveExternalToolBindingRequest } from "../contracts/generated/SaveExternalToolBindingRequest";

export type { ExternalToolBinding, SaveExternalToolBindingRequest };

export function listExternalToolBindings(spaceId: string): Promise<ExternalToolBinding[]> {
  return invoke("plugin:spaces|list_external_tool_bindings", { spaceId });
}

export function saveExternalToolBinding(
  request: SaveExternalToolBindingRequest,
): Promise<ExternalToolBinding> {
  return invoke("plugin:spaces|save_external_tool_binding", { request });
}

export function deleteExternalToolBinding(bindingId: string): Promise<void> {
  return invoke("plugin:spaces|delete_external_tool_binding", { bindingId });
}

export function disableExternalToolBinding(bindingId: string): Promise<void> {
  return invoke("plugin:spaces|disable_external_tool_binding", { bindingId });
}

export function testExternalToolConnection(request: SaveExternalToolBindingRequest): Promise<unknown> {
  return invoke("plugin:spaces|test_external_tool_connection", { request });
}
