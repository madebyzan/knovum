import { computed, type Ref } from "vue";

import { lineageRelationIds } from "../../../model/relations/lineage";
import type { SpaceRelation } from "../../../model/relations/relation";
import type { StructureIndex } from "../../../model/structure/structure-index";
import { contentNodeIdsWithin } from "../../../model/structure/structure-queries";

interface RevealedLineageOptions {
  relations: () => readonly SpaceRelation[];
  nodeIds: Readonly<Ref<ReadonlySet<string>>>;
  structureNodeIds: Readonly<Ref<ReadonlySet<string>>>;
  structureIndex: Readonly<Ref<StructureIndex>>;
}

/**
 * The lineage relations the selection explains. Lineage lines stay hidden at rest and selecting
 * content reveals the chain behind it; a selected group or section explains the content inside
 * it, since pressing grouped content selects its group.
 */
export function useRevealedLineage(options: RevealedLineageOptions) {
  return computed(() => {
    const index = options.structureIndex.value;
    // The selection is reconciled with the structure after it changes, so read what still exists.
    const roots = [...options.structureNodeIds.value].filter((id) => index.structureById.has(id));
    const contentIds = [...options.nodeIds.value, ...contentNodeIdsWithin(roots, index)];
    return lineageRelationIds(options.relations(), contentIds);
  });
}
