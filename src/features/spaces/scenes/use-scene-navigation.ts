import { useTimeoutFn } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";

import { getCanvasEditorSession } from "../canvas/editor-session";
import { useSpaceStore } from "../session/space-store";
import { createSceneNavigationActions, type SceneNavigationContext } from "./scene-navigation-actions";

export function useSceneNavigation() {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const { scenes, activeSpace } = storeToRefs(store);
  const recentSceneId = ref<string>();
  const { start: triggerRecentTimer, stop: stopRecentTimer } = useTimeoutFn(
    () => {
      recentSceneId.value = undefined;
    },
    2200,
    { immediate: false },
  );
  const context: SceneNavigationContext = {
    store,
    scenes,
    activeSpace,
    viewport: editorSession.viewport,
    title: ref(""),
    editingId: ref<string>(),
    editingTitle: ref(""),
    recentSceneId,
    root: ref<HTMLElement>(),
    triggerRecentTimer,
    stopRecentTimer,
  };
  const currentZoom = computed(() => Math.round((editorSession.viewport.value?.zoom ?? 1) * 100));
  const activeSceneId = computed(() => {
    const viewport = editorSession.viewport.value;
    if (!viewport) return undefined;
    return scenes.value.find(
      (scene) =>
        Math.abs(scene.viewportX - viewport.x) < 2 &&
        Math.abs(scene.viewportY - viewport.y) < 2 &&
        Math.abs(scene.viewportZoom - viewport.zoom) < 0.005,
    )?.id;
  });

  const actions = createSceneNavigationActions(context);

  function targetSceneIndex(dir: -1 | 1): number {
    const total = scenes.value.length;
    if (!total) return -1;
    const activeIndex = scenes.value.findIndex((scene) => scene.id === activeSceneId.value);
    if (activeIndex < 0) return dir === 1 ? 0 : total - 1;
    return (activeIndex + dir + total) % total;
  }

  function stepScene(step: -1 | 1, onFallback?: () => void) {
    const target = targetSceneIndex(step);
    const candidate = target >= 0 ? scenes.value[target] : undefined;
    if (candidate) actions.openScene(candidate);
    else onFallback?.();
  }

  function getTargetScene(dir: -1 | 1) {
    const target = targetSceneIndex(dir);
    return target >= 0 ? scenes.value[target] : undefined;
  }

  return {
    scenes,
    title: context.title,
    editingId: context.editingId,
    editingTitle: context.editingTitle,
    recentSceneId: context.recentSceneId,
    root: context.root,
    currentZoom,
    activeSceneId,
    stepScene,
    getTargetScene,
    ...actions,
  };
}
