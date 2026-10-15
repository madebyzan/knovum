import { usePreferredReducedMotion, useTransition } from "@vueuse/core";
import { computed, type Ref } from "vue";

/** The panel clock. `--knovum-panel-motion-*` in tokens.css carries the same values for the CSS fades. */
export const PANEL_MOTION = {
  durationMs: 280,
  easing: [0.2, 0, 0, 1] as [number, number, number, number],
};

interface WorkspaceEdgeState {
  sidebarPinned: Readonly<Ref<boolean>>;
  sidebarVisible: Readonly<Ref<boolean>>;
  sidebarWidth: Readonly<Ref<number>>;
  rightSidebarOpen: Readonly<Ref<boolean>>;
  rightSidebarWidth: Readonly<Ref<number>>;
}

/** Workspace geometry in whole CSS pixels, measured from the window edge each sidebar belongs to. */
export interface WorkspaceEdges {
  /** Width the leading sidebar reserves in layout; the work area starts here, or at the gutter. */
  leading: number;
  /** Where the leading sidebar's own trailing edge is: pinned and previewing both show it in full. */
  sidebar: number;
  /** Width the trailing sidebar reserves in layout. */
  trailing: number;
}

/**
 * Panel motion as one clock. The motion animates panel *state* - pinned, visible, open - not widths, so a
 * sash drag changes a width with no motion to suspend, and everything that lines up with an edge derives it
 * from the same frame value: the reserved widths, the sidebar's position and the toolbar columns.
 *
 * Edges land on whole CSS pixels. The work area's origin moves with the leading edge, and engines place
 * composited layers (the Konva canvases, the grid) on whole pixels while DOM text keeps the fraction, so a
 * fractional origin shifts card text against its own frame by a different sub-pixel amount every frame. A
 * CSS transition cannot be told to land on pixels; this clock can.
 */
export function useWorkspaceEdges(state: WorkspaceEdgeState): Readonly<Ref<WorkspaceEdges>> {
  const reducedMotion = usePreferredReducedMotion();
  const progress = useTransition(
    [
      () => Number(state.sidebarPinned.value),
      () => Number(state.sidebarVisible.value),
      () => Number(state.rightSidebarOpen.value),
    ],
    {
      duration: PANEL_MOTION.durationMs,
      transition: PANEL_MOTION.easing,
      disabled: computed(() => reducedMotion.value === "reduce"),
    },
  );
  return computed(() => {
    const [pinned, visible, rightOpen] = progress.value;
    return {
      leading: Math.round(state.sidebarWidth.value * pinned),
      sidebar: Math.round(state.sidebarWidth.value * visible),
      trailing: Math.round(state.rightSidebarWidth.value * rightOpen),
    };
  });
}
