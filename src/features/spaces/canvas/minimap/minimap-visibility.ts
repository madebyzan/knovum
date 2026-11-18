const MIN_TOGGLE_HOST_WIDTH = 360;
const MIN_TOGGLE_HOST_HEIGHT = 240;
const MIN_PANEL_HOST_WIDTH = 480;
const MIN_PANEL_HOST_HEIGHT = 300;
const MIN_PANEL_HOST_WIDTH_WITH_RECORDING = 704;

export interface MinimapVisibilityInput {
  readonly open: boolean;
  readonly navigationEnabled: boolean;
  readonly hasContent: boolean;
  readonly hostSize: { readonly width: number; readonly height: number };
  readonly recordingDockOpen: boolean;
}

export interface MinimapVisibilityState {
  readonly showToggle: boolean;
  readonly showPanel: boolean;
  readonly panelFitsHost: boolean;
}

/**
 * Resolves whether the mini map toggle and panel can be displayed without colliding with
 * bottom-left view controls or the centered audio recording dock.
 */
export function resolveMinimapVisibility(input: MinimapVisibilityInput): MinimapVisibilityState {
  const { hostSize } = input;
  const showToggle =
    input.navigationEnabled &&
    input.hasContent &&
    hostSize.width >= MIN_TOGGLE_HOST_WIDTH &&
    hostSize.height >= MIN_TOGGLE_HOST_HEIGHT;
  if (!showToggle) {
    return { showToggle: false, showPanel: false, panelFitsHost: false };
  }
  const requiredWidth = input.recordingDockOpen ? MIN_PANEL_HOST_WIDTH_WITH_RECORDING : MIN_PANEL_HOST_WIDTH;
  const panelFitsHost = hostSize.width >= requiredWidth && hostSize.height >= MIN_PANEL_HOST_HEIGHT;
  return {
    showToggle: true,
    showPanel: panelFitsHost && input.open,
    panelFitsHost,
  };
}
