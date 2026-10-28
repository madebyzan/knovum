import { placeActionSurface } from "./selection-action-placement";
import { placeInspectorSurface } from "./selection-inspector-placement";
import { intersection, type PlacementInput, type SurfacePlacement } from "./selection-surface-geometry";

/**
 * Selection chrome depends on selected geometry and fixed canvas controls, never other canvas nodes.
 * It belongs to the selection as a whole: a bar over one member of a multi-selection would read as
 * that member's bar and cover the content being arranged.
 */
export function placeSelectionSurface(input: PlacementInput): SurfacePlacement | undefined {
  const { width, height } = input.surface;
  if (
    ![width, height, input.viewport.width, input.viewport.height].every(
      (value) => Number.isFinite(value) && value > 0,
    )
  )
    return undefined;
  const safe = {
    left: input.insets.left,
    top: input.insets.top,
    right: input.viewport.width - input.insets.right,
    bottom: input.viewport.height - input.insets.bottom,
  };
  const visible = intersection(input.selection, safe);
  if (!visible) return undefined;
  if (input.kind === "inspector") return placeInspectorSurface(input, safe, visible);
  if (input.kind === "compact-editor")
    return placeActionSurface(input, safe, visible) ?? placeInspectorSurface(input, safe, visible);
  return placeActionSurface(input, safe, visible);
}
