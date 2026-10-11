export type SpaceSurface =
  | { kind: "canvas" }
  | { kind: "library" }
  | { kind: "document"; nodeId: string }
  | { kind: "video"; nodeId: string };

export function spaceWorkbenchCapabilities(surface: SpaceSurface, hasSpace: boolean) {
  const canvas = hasSpace && surface.kind === "canvas";
  return { canvas, outline: canvas, rightSidebar: canvas };
}
