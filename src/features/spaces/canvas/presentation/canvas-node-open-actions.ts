import {
  ArrowUpRight,
  Code2,
  ExternalLink,
  FileText,
  Image as ImageIcon,
  Pencil,
  Play,
  Volume2,
} from "@lucide/vue";
import type { Component } from "vue";

import type { SpaceNodeKind } from "../../model/nodes/node-payload";

export interface NodeOpenAction {
  label: string;
  icon: Component;
}

/**
 * Opening is a kind-specific surface, so only the kinds that have one are listed. A kind absent
 * here has nothing to open: text, credential and knowledge-object nodes carry their content on the
 * card itself, and the action bar leads with another action for them.
 */
export const OPEN_ACTION_BY_NODE_KIND: Partial<Record<SpaceNodeKind, NodeOpenAction>> = {
  note: { label: "common.edit", icon: Pencil },
  markdown: { label: "common.edit", icon: Pencil },
  website: { label: "spaces.canvas.nodeActions.openOriginal", icon: ExternalLink },
  portal: { label: "spaces.canvas.nodeActions.goToTarget", icon: ArrowUpRight },
  image: { label: "spaces.canvas.nodeActions.viewLarge", icon: ImageIcon },
  video: { label: "spaces.canvas.nodeActions.openVideo", icon: Play },
  audio: { label: "spaces.canvas.nodeActions.openAudio", icon: Volume2 },
  file: { label: "spaces.canvas.nodeActions.openFile", icon: FileText },
  code: { label: "spaces.canvas.nodeActions.viewCode", icon: Code2 },
};
