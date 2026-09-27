import { defineStore } from "pinia";
import { readonly, shallowRef } from "vue";

import type { CanvasTool } from "./contracts/canvas-tool";
import { flushSpaceSessionContent } from "./editor-session";

export const useCanvasToolStore = defineStore("spaces-canvas-tools", () => {
  const tool = shallowRef<CanvasTool>("select");
  let request = 0;

  async function selectTool(next: CanvasTool) {
    const current = ++request;
    await flushSpaceSessionContent();
    if (current === request) tool.value = next;
  }

  function resetTool() {
    request++;
    tool.value = "select";
  }

  return { tool: readonly(tool), selectTool, resetTool };
});
