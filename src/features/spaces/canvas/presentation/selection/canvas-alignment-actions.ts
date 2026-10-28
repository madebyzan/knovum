import {
  AlignCenterHorizontal,
  AlignCenterVertical,
  AlignEndHorizontal,
  AlignEndVertical,
  AlignHorizontalDistributeCenter,
  AlignStartHorizontal,
  AlignStartVertical,
  AlignVerticalDistributeCenter,
} from "@lucide/vue";
import type { Component } from "vue";

import type { CanvasAlignMode, CanvasDistributeMode } from "../../contracts/canvas-editor-port";

/** Edges in reading order: the three vertical guides, then the three horizontal ones. */
export const CANVAS_ALIGN_ACTIONS: readonly { mode: CanvasAlignMode; icon: Component }[] = [
  { mode: "left", icon: AlignStartVertical },
  { mode: "center", icon: AlignCenterVertical },
  { mode: "right", icon: AlignEndVertical },
  { mode: "top", icon: AlignStartHorizontal },
  { mode: "middle", icon: AlignCenterHorizontal },
  { mode: "bottom", icon: AlignEndHorizontal },
];

export const CANVAS_DISTRIBUTE_ACTIONS: readonly { mode: CanvasDistributeMode; icon: Component }[] = [
  { mode: "horizontal", icon: AlignHorizontalDistributeCenter },
  { mode: "vertical", icon: AlignVerticalDistributeCenter },
];
