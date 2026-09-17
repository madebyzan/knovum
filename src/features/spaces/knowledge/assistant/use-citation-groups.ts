import {
  AudioLines,
  FileCode2,
  FileSpreadsheet,
  FileText,
  Film,
  Globe2,
  ImageIcon,
  Link2,
  Presentation,
  Shapes,
  StickyNote,
} from "@lucide/vue";
import { isUndefined, omitBy } from "es-toolkit";
import { computed, type Component } from "vue";

import { classifyFilePresentation } from "../../assets/file-presentation";
import type { SpaceNodeKind } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import type { GroundedCitation } from "../model/grounded-citation";

export interface CitationGroup {
  key: string;
  title: string;
  space?: string;
  kind?: SpaceNodeKind;
  fileName?: string;
  citations: Array<{ citation: GroundedCitation; index: number }>;
}

const KIND_ICONS: Partial<Record<SpaceNodeKind, Component>> = {
  audio: AudioLines,
  code: FileCode2,
  image: ImageIcon,
  note: StickyNote,
  object: Shapes,
  portal: Link2,
  video: Film,
  website: Globe2,
};

export function useCitationGroups(citations: () => readonly GroundedCitation[]) {
  const store = useSpaceStore();

  const groups = computed(() => {
    const bySource = new Map<string, CitationGroup>();
    for (const citation of citations()) {
      const key =
        citation.state === "current" ? citation.nodeId : `${citation.sourceRevision}:${citation.nodeTitle}`;
      const group = bySource.get(key) ?? createGroup(key, citation, store);
      group.citations.push({ citation, index: citation.index });
      bySource.set(key, group);
    }
    return [...bySource.values()];
  });

  function iconFor(group: CitationGroup): Component {
    if (group.kind === "file") {
      const presentation = classifyFilePresentation(group.fileName ?? group.title);
      if (presentation === "spreadsheet" || presentation === "tabular-data") {
        return FileSpreadsheet;
      }
      if (presentation === "presentation") return Presentation;
    }
    return (group.kind && KIND_ICONS[group.kind]) || FileText;
  }

  return { groups, iconFor };
}

function createGroup(
  key: string,
  citation: GroundedCitation,
  store: ReturnType<typeof useSpaceStore>,
): CitationGroup {
  const node =
    citation.state === "current" ? store.nodes.find((item) => item.id === citation.nodeId) : undefined;
  const fileName = node && "fileName" in node.data ? node.data.fileName : undefined;
  return omitBy(
    {
      key,
      title: citation.nodeTitle,
      space: citation.sourceSpaceTitle,
      kind: node?.data.kind,
      fileName,
      citations: [],
    },
    isUndefined,
  ) as CitationGroup;
}
