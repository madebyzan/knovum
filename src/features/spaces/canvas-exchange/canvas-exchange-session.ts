import { readonly, shallowRef } from "vue";

import type { CanvasExchangePort } from "./canvas-exchange-port";

const activePort = shallowRef<CanvasExchangePort>();

export function useCanvasExchangeSession() {
  return { activePort: readonly(activePort) };
}

export function registerCanvasExchangePort(port: CanvasExchangePort) {
  activePort.value = port;
  return () => {
    if (activePort.value === port) activePort.value = undefined;
  };
}
