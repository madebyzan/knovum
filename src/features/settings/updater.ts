export type { UpdateMetadata } from "../../shared/contracts/generated/UpdateMetadata";
import type { UpdateMetadata } from "../../shared/contracts/generated/UpdateMetadata";
export type { UpdateConfiguration } from "../../shared/contracts/generated/UpdateConfiguration";
import { invokeCommand } from "../../foundation/ipc/invoke-command";
import type { UpdateConfiguration } from "../../shared/contracts/generated/UpdateConfiguration";

export function getUpdateConfiguration() {
  return invokeCommand<UpdateConfiguration>("get_update_configuration");
}

export function checkForUpdate() {
  return invokeCommand<UpdateMetadata | null>("check_for_update");
}

export function installUpdate() {
  return invokeCommand("install_update");
}
