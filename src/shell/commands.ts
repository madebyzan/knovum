import type { AppCommand } from "../shared/contracts/app-command";
import type { WorkspaceDefinition } from "../shared/contracts/workspace-definition";

interface ShellCommandActions {
  openCommandPalette: () => void;
  openSettings: () => void;
  openShortcuts?: () => void;
  toggleSidebar: () => void;
  toggleInspector: () => void;
}

interface ShellCommandLabels {
  settings: string;
  commandPalette: string;
  shortcuts?: string;
  toggleSidebar: string;
  toggleInspector: string;
}

interface ShellCommandContext {
  actions: ShellCommandActions;
  labels: ShellCommandLabels;
}

export function createShellCommands(
  workspace: WorkspaceDefinition,
  workspaceActive: boolean,
  context: ShellCommandContext,
): readonly AppCommand[] {
  const { actions, labels } = context;
  const viewCommands: AppCommand[] = [];
  if (workspaceActive) {
    viewCommands.push({
      id: "view.toggle-sidebar",
      title: labels.toggleSidebar,
      shortcut: "$mod+B",
      category: "navigation",
      run: actions.toggleSidebar,
    });
    viewCommands.push({
      id: "view.toggle-inspector",
      title: labels.toggleInspector,
      category: "navigation",
      run: actions.toggleInspector,
    });
  }

  const baseCommands: AppCommand[] = [
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

  return [...baseCommands, ...(workspaceActive ? workspace.commands.value : []), ...viewCommands];
}
