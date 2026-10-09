import { error as logError } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import { canRunAction } from "../../shared/contracts/action-availability";
import type { AppCommand } from "../../shared/contracts/app-command";
import { isApplicationLeaveCancelled } from "../../shared/contracts/application-leave";
import { isEditableTarget } from "../dom/editable-target";
import { formatErrorDiagnostic, getErrorMessage } from "../errors/error-format";
import { isApplicationTransitioning } from "../lifecycle/application-transition";

let registeredCommands: readonly AppCommand[] = [];

export function registerAppCommands(commands: readonly AppCommand[]): () => void {
  registeredCommands = commands;
  return () => {
    if (registeredCommands === commands) registeredCommands = [];
  };
}

export function getAppCommand(commandId: string): AppCommand | undefined {
  return registeredCommands.find((command) => command.id === commandId);
}

export function runAppCommandById(commandId: string): void {
  const command = getAppCommand(commandId);
  if (command) runAppCommand(command);
}

export function runAppCommand(command: AppCommand): void {
  if (isApplicationTransitioning()) return;
  if (!modalScopeAllows(command.id)) return;
  if (command.availability && !canRunAction(command.availability())) return;
  void Promise.resolve()
    .then(() => (isApplicationTransitioning() || !modalScopeAllows(command.id) ? undefined : command.run()))
    .catch((error) => {
      if (isApplicationLeaveCancelled(error)) return;
      void logError(`Command ${command.id} failed: ${formatErrorDiagnostic(error)}`);
      toast.error(getErrorMessage(error));
    });
}

function modalScopeAllows(commandId: string): boolean {
  const scopes = document.querySelectorAll<HTMLElement>("[data-app-command-scope][data-state='open']");
  const activeScope = scopes.item(scopes.length - 1);
  if (!activeScope) return true;
  const focused = document.activeElement;
  if (
    commandId.startsWith("editing.") &&
    focused &&
    activeScope.contains(focused) &&
    isEditableTarget(focused)
  )
    return true;
  const allowed = activeScope.dataset.appCommandScope?.split(" ").filter(Boolean) ?? [];
  return allowed.includes(commandId);
}
