import {
  ArrowRightLeft,
  AudioLines,
  Captions,
  Code2,
  File,
  FileText,
  Film,
  Globe2,
  Image,
  KeyRound,
  LayoutGrid,
  MessageSquareQuote,
  StickyNote,
  Type,
} from "@lucide/vue";

import type { SpaceNodeKind } from "../model/nodes/node-payload";

const NODE_ICONS = {
  website: Globe2,
  markdown: FileText,
  note: StickyNote,
  text: Type,
  audio: AudioLines,
  credential: KeyRound,
  code: Code2,
  image: Image,
  video: Film,
  subtitle: Captions,
  object: MessageSquareQuote,
  portal: ArrowRightLeft,
  section: LayoutGrid,
  file: File,
} satisfies Record<SpaceNodeKind, typeof File>;

export function nodeIcon(kind: SpaceNodeKind | string) {
  return (NODE_ICONS as Record<string, typeof File>)[kind] ?? File;
}
