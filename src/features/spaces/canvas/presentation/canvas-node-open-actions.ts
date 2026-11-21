import { ArrowUpRight, ExternalLink, Eye, Pencil } from "@lucide/vue";
import type { Component } from "vue";

import { hasSystemQuickLook, systemPreviewLabelKey } from "../../assets/asset-preview";
import type { SpaceNodeKind } from "../../model/nodes/node-payload";

export interface NodeOpenAction {
  label: string;
  icon: Component;
}

const FIXED_OPEN_ACTION_BY_KIND: Partial<Record<SpaceNodeKind, NodeOpenAction>> = {
  note: { label: "common.edit", icon: Pencil },
  markdown: { label: "common.edit", icon: Pencil },
  website: { label: "spaces.canvas.nodeActions.openOriginal", icon: ExternalLink },
  portal: { label: "spaces.canvas.nodeActions.goToTarget", icon: ArrowUpRight },
};

const SYSTEM_PREVIEW_NODE_KINDS = new Set<SpaceNodeKind>(["image", "video", "audio", "file", "code"]);

/**
 * Resolves the primary open action for a canvas node kind. Managed assets use macOS Quick Look on
 * macOS and open in the default associated application on Windows/Linux, so both label and icon
 * match the platform behavior. Kinds absent here lead with Ask Knovum instead.
 */
export function resolveNodeOpenAction(kind: SpaceNodeKind): NodeOpenAction | undefined {
  const fixed = FIXED_OPEN_ACTION_BY_KIND[kind];
  if (fixed) return fixed;
  if (!SYSTEM_PREVIEW_NODE_KINDS.has(kind)) return undefined;
  return {
    label: systemPreviewLabelKey(),
    icon: hasSystemQuickLook() ? Eye : ArrowUpRight,
  };
}
