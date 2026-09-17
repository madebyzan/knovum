import { invoke } from "@tauri-apps/api/core";

export interface UpdaterStatus {
  configured: boolean;
}

export interface UpdateMetadata {
  version: string;
  currentVersion: string;
}

export function getUpdaterStatus() {
  return invoke<UpdaterStatus>("updater_get_status");
}

export function checkForUpdate() {
  return invoke<UpdateMetadata | null>("updater_check");
}

export function installUpdate() {
  return invoke("updater_install");
}
