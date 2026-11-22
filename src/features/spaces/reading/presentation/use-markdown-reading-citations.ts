import { computed, shallowRef, watch, type Ref } from "vue";

import type { MarkdownCitations } from "../../../../shared/ui/markdown-renderer";
import { useCitationHover } from "../../conversation/presentation/use-citation-hover";
import { loadGroundedDocumentMetadata } from "../../knowledge-derivations/document/metadata";
import { findGroundedCitation, groundedCitationPeekTarget } from "../../knowledge/model/grounded-citation";
import type { GroundedCitation } from "../../knowledge/model/schema/grounded-citation";
import { useSpaceStore } from "../../session/space-store";

const EMPTY_CITATIONS: MarkdownCitations = new Map();

/**
 * Loads grounded citation provenance for a Markdown document shown in the reader so grounded
 * `[Cn]` markers render as superscript badges with hover previews and click-to-navigate into the
 * Source Reader history stack, while non-grounded documents stay completely untouched.
 */
export function useMarkdownReadingCitations(nodeId: Ref<string>, isMarkdown: Ref<boolean>) {
  const store = useSpaceStore();
  const citations = shallowRef<readonly GroundedCitation[]>([]);
  const hover = useCitationHover(citations);
  let requestNonce = 0;

  watch(
    [nodeId, isMarkdown],
    async ([id, markdown]) => {
      hover.dismiss();
      requestNonce += 1;
      const current = requestNonce;
      if (!markdown) {
        citations.value = [];
        return;
      }
      try {
        const metadata = await loadGroundedDocumentMetadata(id);
        if (current !== requestNonce) return;
        citations.value = metadata?.citations ?? [];
      } catch {
        if (current !== requestNonce) return;
        citations.value = [];
      }
    },
    { immediate: true },
  );

  const citationMap = computed<MarkdownCitations>(() => {
    if (!citations.value.length) return EMPTY_CITATIONS;
    return new Map(citations.value.map((item) => [item.index, item.state === "current"]));
  });

  function openCitation(citation: GroundedCitation | undefined) {
    hover.dismiss();
    const target = groundedCitationPeekTarget(citation);
    if (target) store.openSource(target);
  }

  function openCitationIndex(index: number) {
    openCitation(findGroundedCitation(citations.value, index));
  }

  return {
    citations,
    citationMap,
    hover,
    openCitation,
    openCitationIndex,
  };
}
