import type { AppCommand } from "../../shared/contracts/app-command";
import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { EDITING_COMMANDS, runEditingCommand } from "./editing-command";

interface ShellCommandActions {
  openCommandPalette: () => void;
  openSettings: () => void;
  openShortcuts?: () => void;
  toggleSidebar: () => void;
  toggleRightSidebar: () => void;
}

interface ShellCommandLabels {
  editingLabelByCommand: Record<EditingCommand, string>;
  settings: string;
  commandPalette: string;
  shortcuts?: string;
  toggleSidebar: string;
  toggleRightSidebar: string;
}

interface ShellCommandContext {
  actions: ShellCommandActions;
  labels: ShellCommandLabels;
  workspaceCommands: readonly AppCommand[];
}

export function createShellCommands(
  workspace: WorkspaceDefinition,
  workspaceActive: boolean,
  context: ShellCommandContext,
): readonly AppCommand[] {
  const { actions, labels, workspaceCommands } = context;
  const viewCommands: AppCommand[] = [];
  if (workspaceActive) {
    viewCommands.push({
      id: "view.toggle-sidebar",
      title: labels.toggleSidebar,
      shortcut: "$mod+B",
      category: "navigation",
      run: actions.toggleSidebar,
    });
    if (workspace.rightSidebarAvailable.value) {
      viewCommands.push({
        id: "view.toggle-right-sidebar",
        title: labels.toggleRightSidebar,
        shortcut: "$mod+Shift+B",
        allowInEditable: true,
        category: "navigation",
        run: actions.toggleRightSidebar,
      });
    }
  }

  const baseCommands: AppCommand[] = [
    // The Application Menu owns the chord for these, so they carry no keyboard binding of their own.
    ...EDITING_COMMANDS.map<AppCommand>((intent) => ({
      id: `editing.${intent}`,
      title: labels.editingLabelByCommand[intent],
      category: "editing",
      palette: false,
      run: () => runEditingCommand(intent),
    })),
    {
      id: "shell.command-palette",
      title: labels.commandPalette,
      shortcut: "$mod+K",
      appMenuShortcut: true,
      allowInEditable: true,
      category: "general",
      run: actions.openCommandPalette,
    },
    {
      id: "navigate.settings",
      title: labels.settings,
      shortcut: "$mod+,",
      appMenuShortcut: true,
      category: "general",
      run: actions.openSettings,
    },
  ];

  if (actions.openShortcuts && labels.shortcuts) {
    baseCommands.push({
      id: "shell.shortcuts",
      title: labels.shortcuts,
      shortcut: "$mod+/",
      category: "general",
      run: actions.openShortcuts,
    });
  }

  return [...baseCommands, ...(workspaceActive ? workspaceCommands : []), ...viewCommands];
}
