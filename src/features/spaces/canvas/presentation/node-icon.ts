import {
  ArrowRightLeft,
  AudioLines,
  Code2,
  File,
  FileText,
  Film,
  Globe2,
  Image,
  KeyRound,
  MessageSquareQuote,
  StickyNote,
  Type,
} from "@lucide/vue";

import type { SpaceNodeKind } from "../../model/nodes/node-payload";

const ICON_BY_NODE_KIND = {
  website: Globe2,
  markdown: FileText,
  note: StickyNote,
  text: Type,
  audio: AudioLines,
  credential: KeyRound,
  code: Code2,
  image: Image,
  video: Film,
  object: MessageSquareQuote,
  portal: ArrowRightLeft,
  file: File,
} satisfies Record<SpaceNodeKind, typeof File>;

export function nodeIcon(kind: string) {
  return (ICON_BY_NODE_KIND as Record<string, typeof File>)[kind] ?? File;
}
