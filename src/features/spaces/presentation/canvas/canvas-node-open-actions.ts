import {
  ArrowUpRight,
  Code2,
  ExternalLink,
  FileText,
  Image as ImageIcon,
  Info,
  Pencil,
  Play,
  Volume2,
} from "@lucide/vue";
import type { Component } from "vue";

import type { SpaceNodeKind } from "../../model/nodes/node-payload";

export const nodeOpenLabels = {
  note: "common.edit",
  text: "spaces.inspector.title",
  markdown: "common.edit",
  website: "spaces.canvas.nodeActions.openOriginal",
  portal: "spaces.canvas.nodeActions.goToTarget",
  image: "spaces.canvas.nodeActions.viewLarge",
  video: "spaces.canvas.nodeActions.openVideo",
  audio: "spaces.canvas.nodeActions.openAudio",
  file: "spaces.canvas.nodeActions.openFile",
  code: "spaces.canvas.nodeActions.viewCode",
  credential: "spaces.inspector.title",
  object: "spaces.inspector.title",
} satisfies Record<SpaceNodeKind, string>;

export const nodeOpenIcons = {
  note: Pencil,
  text: Info,
  markdown: Pencil,
  website: ExternalLink,
  portal: ArrowUpRight,
  image: ImageIcon,
  video: Play,
  audio: Volume2,
  file: FileText,
  code: Code2,
  credential: Info,
  object: Info,
} satisfies Record<SpaceNodeKind, Component>;
