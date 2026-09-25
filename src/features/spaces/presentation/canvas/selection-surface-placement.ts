import { placeActionSurface } from "./selection-action-placement";
import { placeInspectorSurface } from "./selection-inspector-placement";
import {
  intersection,
  type PlacementInput,
  type SurfacePlacement,
  type SurfaceRect,
} from "./selection-surface-geometry";

function actionAnchor(input: PlacementInput, safe: SurfaceRect, visible: SurfaceRect): SurfaceRect {
  if (input.members.length < 2) return input.selection;
  const center = { x: (visible.left + visible.right) / 2, y: (visible.top + visible.bottom) / 2 };
  const members = input.members.flatMap((member) => {
    const clipped = intersection(member, safe);
    return clipped ? [clipped] : [];
  });
  const centeredMember = members.find(
    (member) =>
      center.x >= member.left &&
      center.x <= member.right &&
      center.y >= member.top &&
      center.y <= member.bottom,
  );
  if (centeredMember) return centeredMember;
  const distance = (rect: SurfaceRect) =>
    Math.hypot((rect.left + rect.right) / 2 - center.x, (rect.top + rect.bottom) / 2 - center.y);
  return members.toSorted((a, b) => distance(a) - distance(b))[0] ?? input.selection;
}

/** Selection chrome depends on selected geometry and fixed canvas controls, never other canvas nodes. */
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
  const selection = actionAnchor(input, safe, visible);
  return placeActionSurface({ ...input, selection }, safe, intersection(selection, safe) ?? visible);
}
