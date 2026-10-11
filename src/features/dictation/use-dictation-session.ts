import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { readonly, shallowRef } from "vue";

import { invokeCommand } from "../../foundation/ipc/invoke-command";
import type { DictationSnapshot } from "./contracts/generated/DictationSnapshot";

const STATE_EVENT = "knovum://dictation-state";
const IDLE_DICTATION_SNAPSHOT: DictationSnapshot = {
  phase: "idle",
  resources: null,
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
  await invokeCommand("plugin:dictation|dismiss_transcription");
}

async function copy() {
  await invokeCommand("plugin:dictation|copy_transcription");
}

export function useDictationSession() {
  const state = shallowRef<DictationSnapshot>(IDLE_DICTATION_SNAPSHOT);
  let unlisten: UnlistenFn | undefined;
  let epoch = 0;

  async function start() {
    const owner = ++epoch;
    let eventVersion = 0;
    const release = await listen<DictationSnapshot>(STATE_EVENT, ({ payload }) => {
      if (owner !== epoch) return;
      eventVersion += 1;
      state.value = payload;
    });
    if (owner !== epoch) {
      release();
      return;
    }
    unlisten?.();
    unlisten = release;
    const version = eventVersion;
    try {
      const snapshot = await invokeCommand<DictationSnapshot>("plugin:dictation|get_state");
      if (owner === epoch && eventVersion === version) state.value = snapshot;
    } catch (error) {
      if (owner === epoch) stop();
      throw error;
    }
  }

  function stop() {
    epoch += 1;
    unlisten?.();
    unlisten = undefined;
  }

  return { state: readonly(state), start, stop, cancel, dismiss, copy };
}
