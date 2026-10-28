import { defineStore } from "pinia";
import { readonly, shallowRef } from "vue";

import { flushSpaceSessionContent } from "../workspace/session-participants";
import type { CanvasTool } from "./contracts/canvas-tool";

export const useCanvasToolStore = defineStore("spaces-canvas-tools", () => {
  const tool = shallowRef<CanvasTool>("select");
  let request = 0;

  async function selectTool(next: CanvasTool) {
    const current = ++request;
    await flushSpaceSessionContent({ spaceIds: [] });
    if (current === request) tool.value = next;
  }

  function resetTool() {
    request++;
    tool.value = "select";
  }

  return { tool: readonly(tool), selectTool, resetTool };
});
