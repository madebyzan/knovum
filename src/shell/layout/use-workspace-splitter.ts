import type { SplitterPanel } from "reka-ui";
import { shallowRef, useTemplateRef, watch } from "vue";

interface WorkspaceSplitterOptions {
  sidebarPinned(): boolean;
  sidebarWidth(): number;
  rightSidebarOpen(): boolean;
  rightSidebarWidth(): number;
  onLayout(sizes: number[]): void;
}

export function useWorkspaceSplitter(options: WorkspaceSplitterOptions) {
  const leadingPanel = useTemplateRef<InstanceType<typeof SplitterPanel>>("leadingPanel");
  const trailingPanel = useTemplateRef<InstanceType<typeof SplitterPanel>>("trailingPanel");
  const layoutReady = shallowRef(false);

  watch(
    () => [options.sidebarPinned(), layoutReady.value] as const,
    ([pinned, ready]) => {
      if (!ready) return;
      if (pinned) leadingPanel.value?.resize(options.sidebarWidth());
      else leadingPanel.value?.collapse();
    },
    { flush: "post" },
  );
  watch(
    () => [options.rightSidebarOpen(), layoutReady.value] as const,
    ([open, ready]) => {
      if (!ready) return;
      if (open) trailingPanel.value?.resize(options.rightSidebarWidth());
      else trailingPanel.value?.collapse();
    },
    { flush: "post" },
  );

  function handleLayout(sizes: number[]) {
    if (sizes.length !== 3) return;
    options.onLayout(sizes);
    if (!layoutReady.value) layoutReady.value = true;
  }

  return { handleLayout, leadingPanel, trailingPanel };
}
