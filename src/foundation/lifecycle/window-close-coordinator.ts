import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

type WindowCloseCommit = () => Promise<void>;

const commits = new Set<WindowCloseCommit>();
let listener: Promise<() => void> | undefined;
let exitListener: Promise<() => void> | undefined;
let closing = false;
let exiting = false;

export async function initializeApplicationCloseCoordinator() {
  listener ??= attachWindowCloseListener();
  exitListener ??= attachApplicationExitListener();
  await Promise.all([listener, exitListener]);
  await invoke("plugin:desktop|register_exit_coordinator");
}

export function registerWindowCloseCommit(commit: WindowCloseCommit) {
  commits.add(commit);
  void initializeApplicationCloseCoordinator().catch((error) =>
    logError(`Application close coordinator initialization failed: ${String(error)}`),
  );
  return () => commits.delete(commit);
}

async function attachApplicationExitListener() {
  return listen("knovum://app-exit-requested", async () => {
    if (exiting) return;
    exiting = true;
    try {
      await commitAll();
      await invoke("plugin:desktop|resolve_app_exit", { approved: true });
    } catch (error) {
      await logError(`Application exit commit failed: ${String(error)}`);
      await invoke("plugin:desktop|resolve_app_exit", { approved: false });
    } finally {
      exiting = false;
    }
  });
}

function commitAll() {
  return Promise.all([...commits].map((commit) => commit())).then(() => undefined);
}

async function attachWindowCloseListener() {
  return getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    if (closing) return;
    closing = true;
    try {
      await commitAll();
      await getCurrentWindow().hide();
    } catch (error) {
      await logError(`Window close commit failed: ${String(error)}`);
    } finally {
      closing = false;
    }
  });
}
