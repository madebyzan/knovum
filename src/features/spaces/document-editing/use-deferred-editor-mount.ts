import { nextTick, onScopeDispose, shallowRef, watch, type Ref } from "vue";

type FrameScheduler = (callback: FrameRequestCallback) => number;

export function afterNextPaint(schedule: FrameScheduler = globalThis.requestAnimationFrame) {
  return new Promise<void>((resolve) => {
    schedule(() => schedule(() => resolve()));
  });
}

export function useDeferredEditorMount(editorId: Ref<string | undefined>) {
  const mountedEditorId = shallowRef<string>();
  let requestRevision = 0;

  watch(
    editorId,
    async (id) => {
      const revision = ++requestRevision;
      mountedEditorId.value = undefined;
      if (!id) return;
      await nextTick();
      await afterNextPaint();
      if (revision === requestRevision && editorId.value === id) mountedEditorId.value = id;
    },
    { immediate: true },
  );
  onScopeDispose(() => {
    requestRevision += 1;
  });

  return mountedEditorId;
}
