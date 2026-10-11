import { computed, ref, shallowRef } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { Space, SpaceLibraryScope } from "../model/space";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import type { SpaceScene } from "../scenes/scene";
import type { SpaceUrlCaptureProjection } from "../sources/url-capture-projection";

export function createSpaceContentState() {
  const spaces = ref<Space[]>([]);
  const draftSpace = ref<Space>();
  const activeSpaceId = ref<string>();
  const nodes = shallowRef<SpaceCanvasNode[]>([]);
  const editSnapshot = shallowRef<SpaceEditSnapshot>({
    editRevision: 0,
    structure: [],
    layouts: [],
    relationStyles: [],
  });
  const groupScopePath = shallowRef<string[]>([]);
  const urlCaptures = shallowRef<SpaceUrlCaptureProjection[]>([]);
  const scenes = ref<SpaceScene[]>([]);
  const relations = ref<SpaceRelation[]>([]);
  const ready = ref(false);
  const loading = ref(false);
  const libraryScope = ref<SpaceLibraryScope>("all");
  const libraryLayout = ref<"grid" | "list">("grid");
  const libraryQuery = ref("");
  const librarySearchNonce = ref(0);
  const activeSpace = computed(
    () =>
      spaces.value.find((space) => space.id === activeSpaceId.value) ??
      (draftSpace.value?.id === activeSpaceId.value ? draftSpace.value : undefined),
  );

  return {
    spaces,
    draftSpace,
    activeSpaceId,
    nodes,
    editSnapshot,
    groupScopePath,
    urlCaptures,
    scenes,
    relations,
    ready,
    loading,
    libraryScope,
    libraryLayout,
    libraryQuery,
    librarySearchNonce,
    activeSpace,
  };
}
