import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

import { isApplicationLeaveCancelled } from "../../shared/contracts/application-leave";
import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";
import { isApplicationTransitioning, runApplicationTransition } from "./application-transition";
import { startRequestHeartbeat } from "./request-heartbeat";

interface WindowCloseCoordination {
  flush(): Promise<void>;
  reportFailure(error: unknown, retry?: () => void): void;
}

let handlingClose = false;

export function attachWindowCloseListener(coordination: WindowCloseCoordination) {
  return getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    if (handlingClose) return;
    handlingClose = true;
    try {
      await handleWindowClose(coordination);
    } finally {
      handlingClose = false;
    }
  });
}

async function handleWindowClose(coordination: WindowCloseCoordination) {
  let requestId: number;
  try {
    requestId = await invokeCommand<number>("plugin:desktop|begin_window_close");
  } catch (error) {
    await logError(`Window close could not begin: ${formatErrorDiagnostic(error)}`);
    coordination.reportFailure(error, retryWindowClose);
    return;
  }
  if (isApplicationTransitioning()) {
    await completeWindowClose(requestId, false);
    return;
  }
  const stopHeartbeat = startRequestHeartbeat(() => acknowledgeWindowClose(requestId));
  try {
    await runApplicationTransition(async () => {
      await coordination.flush();
      await invokeCommand("plugin:desktop|complete_window_close", { requestId, hide: true });
    });
  } catch (error) {
    if (!isApplicationLeaveCancelled(error)) {
      await logError(`Window close commit failed: ${formatErrorDiagnostic(error)}`);
      coordination.reportFailure(error, retryWindowClose);
    }
    await completeWindowClose(requestId, false);
  } finally {
    stopHeartbeat();
  }
}

async function acknowledgeWindowClose(requestId: number) {
  try {
    await invokeCommand("plugin:desktop|acknowledge_window_close", { requestId });
    return true;
  } catch (error) {
    await logError(`Window close acknowledgement failed: ${formatErrorDiagnostic(error)}`);
    return false;
  }
}

async function completeWindowClose(requestId: number, hide: boolean) {
  try {
    await invokeCommand("plugin:desktop|complete_window_close", { requestId, hide });
  } catch (error) {
    await logError(`Window close completion failed: ${formatErrorDiagnostic(error)}`);
  }
}

function retryWindowClose() {
  void getCurrentWindow().close();
}
