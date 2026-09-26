export type { UpdateMetadata } from "../../shared/contracts/generated/UpdateMetadata";
import type { UpdateMetadata } from "../../shared/contracts/generated/UpdateMetadata";
export type { UpdaterStatus } from "../../shared/contracts/generated/UpdaterStatus";
import { invokeCommand as invoke } from "../../foundation/ipc/invoke-command";
import type { UpdaterStatus } from "../../shared/contracts/generated/UpdaterStatus";

export function getUpdaterStatus() {
  return invoke<UpdaterStatus>("updater_get_status");
}

export function checkForUpdate() {
  return invoke<UpdateMetadata | null>("updater_check");
}

export function installUpdate() {
  return invoke("updater_install");
}
