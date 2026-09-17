import { getCurrentWindow } from "@tauri-apps/api/window";

let stopWindowActivity: (() => void) | undefined;

export async function installWindowActivityState(): Promise<void> {
  const currentWindow = getCurrentWindow();
  const root = document.documentElement;
  const apply = (focused: boolean) => {
    root.dataset.windowActive = focused ? "true" : "false";
  };

  apply(await currentWindow.isFocused());
  stopWindowActivity?.();
  stopWindowActivity = await currentWindow.onFocusChanged(({ payload }) => apply(payload));
}

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopWindowActivity?.();
    stopWindowActivity = undefined;
  });
}
