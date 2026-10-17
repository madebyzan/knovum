import { camelCase } from "es-toolkit";

import { i18n } from "../../../../foundation/i18n/instance";

/**
 * Actions that carry a human label. The Agent-only queries and mutations of the catalog
 * (`read-node`, `search-knowledge`, `edit-space-structure`, …) are deliberately absent: they name
 * themselves in diagnostic surfaces and never appear in a creation menu.
 */
const LABELED_ACTIONS = new Set([
  "add-video-subtitles",
  "convert-document-to-docx",
  "convert-document-to-html",
  "convert-document-to-pdf",
  "convert-document-to-txt",
  "draft-document",
  "draft-script",
  "draft-social-post",
  "extract-image-selection",
  "generate-image",
  "generate-image-angle",
  "generate-image-depth",
  "generate-video",
  "generate-visualization",
  "remove-image-background",
  "remove-image-object",
  "render-presentation",
  "split-image-layers",
  "summarize-meeting",
  "upscale-image",
]);

/**
 * The single label source for Space Actions. Creation menus, Agent activity and any other surface
 * that names an Action read from here, so one translation renames it everywhere.
 */
export function spaceActionLabel(actionId: string): string {
  if (!LABELED_ACTIONS.has(actionId)) return actionId;
  return i18n.global.t(`spaces.creation.actions.${camelCase(actionId)}`);
}
