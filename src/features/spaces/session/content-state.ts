import { ref, shallowRef } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { Space, SpaceLibraryScope } from "../model/space";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import type { SpaceScene } from "../scenes/scene";

export function createSpaceContentState() {
  const spaces = ref<Space[]>([]);
  const draftSpace = ref<Space>();
  const nodes = shallowRef<SpaceCanvasNode[]>([]);
  const editSnapshot = shallowRef<SpaceEditSnapshot>({
    editRevision: 0,
    structure: [],
    layouts: [],
    relationStyles: [],
  });
  const groupScopePath = shallowRef<string[]>([]);
  const scenes = ref<SpaceScene[]>([]);
  const relations = ref<SpaceRelation[]>([]);
  const ready = ref(false);
  const loading = ref(false);
  const libraryScope = ref<SpaceLibraryScope>("all");
  const libraryLayout = ref<"grid" | "list">("grid");
  const libraryQuery = ref("");
  const librarySearchNonce = ref(0);

  return {
    spaces,
    draftSpace,
    nodes,
    editSnapshot,
    groupScopePath,
    scenes,
    relations,
    ready,
    loading,
    libraryScope,
    libraryLayout,
    libraryQuery,
    librarySearchNonce,
  };
}
