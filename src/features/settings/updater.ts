import { invokeCommand } from "../../foundation/ipc/invoke-command";
import type { UpdateConfiguration } from "../../shared/contracts/generated/UpdateConfiguration";
import type { UpdateMetadata } from "../../shared/contracts/generated/UpdateMetadata";

export function getUpdateConfiguration() {
  return invokeCommand<UpdateConfiguration>("get_update_configuration");
}

export function checkForUpdate() {
  return invokeCommand<UpdateMetadata | null>("check_for_update");
}

export function installUpdate() {
  return invokeCommand("install_update");
}
