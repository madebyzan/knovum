export const POPUP_ANIMATE_CLASSES =
  "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0 motion-reduce:animate-none";

/**
 * Stacking bands for portalled dialog surfaces. Portalled content is inserted at the teleport
 * anchor its component created on mount, so DOM order follows mount order rather than open order:
 * an equal band would leave a confirmation buried under the dialog that raised it. A confirmation
 * decides the fate of that surface, so it owns the band above every dialog.
 */
export const DIALOG_LAYER_CLASSES = {
  dialog: { overlay: "z-40", content: "z-50" },
  confirmation: { overlay: "z-80", content: "z-90" },
} as const;

export type DialogLayer = keyof typeof DIALOG_LAYER_CLASSES;

export const POPOVER_SURFACE_CLASSES =
  "rounded-panel border border-stroke-strong material-overlay shadow-popover";

export const SUBMENU_CONTENT_CLASSES =
  "material-overlay z-50 min-w-48 rounded-panel p-1 shadow-popover ring-1 ring-stroke-default";

export const MENU_ITEM_BASE_CLASSES =
  "flex h-interaction cursor-default items-center gap-2 rounded-md text-label transition-colors outline-none data-disabled:opacity-40 data-highlighted:bg-surface-control-hover";

export const INPUT_BASE_CLASSES =
  "rounded-md border border-stroke-default bg-surface-raised text-fg-primary transition-colors duration-100 ease-standard outline-none placeholder:text-fg-tertiary hover:border-stroke-strong focus:border-stroke-focus focus-visible:outline-none disabled:opacity-40";
