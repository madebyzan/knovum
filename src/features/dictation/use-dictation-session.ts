import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { readonly, shallowRef } from "vue";

import { invokeCommand } from "../../foundation/ipc/invoke-command";
import type { DictationSnapshot } from "./dictation-contracts";

const STATE_EVENT = "knovum://dictation-state";
const idle: DictationSnapshot = {
  phase: "idle",
  sessionId: null,
  startedAt: null,
  targetApplication: null,
  text: null,
  language: null,
  insertion: null,
  errorCode: null,
  errorMessage: null,
};

async function cancel() {
  await invokeCommand("plugin:dictation|cancel");
}

async function dismiss() {
  await invokeCommand("plugin:dictation|dismiss_result");
}

async function copy() {
  await invokeCommand("plugin:dictation|copy_result");
}

export function useDictationSession() {
  const state = shallowRef<DictationSnapshot>(idle);
  let unlisten: UnlistenFn | undefined;

  async function start() {
    unlisten = await listen<DictationSnapshot>(STATE_EVENT, ({ payload }) => {
      state.value = payload;
    });
    state.value = await invokeCommand<DictationSnapshot>("plugin:dictation|get_state");
  }

  function stop() {
    unlisten?.();
    unlisten = undefined;
  }

  return { state: readonly(state), start, stop, cancel, dismiss, copy };
}
