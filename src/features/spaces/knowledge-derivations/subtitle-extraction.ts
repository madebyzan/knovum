import { error as logError } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { useCanvasEditorSession } from "../canvas/editor-session";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceSubtitlePayload } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import { yieldToCanvas } from "./canvas-yield";
import { beginPendingDerivation, endPendingDerivation } from "./pending-session";
import { derivedNodePosition } from "./placement";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export function canCreateSubtitleResource(node: SpaceCanvasNode | undefined): boolean {
  if (node?.data.kind !== "video" || !node.data.sourceRevision) return false;
  const knowledge = node.data.mediaKnowledge;
  return (
    knowledge?.schemaVersion === 4 &&
    knowledge.sourceRevision === node.data.sourceRevision &&
    knowledge.transcript.subtitleCues.length > 0
  );
}

export async function createSubtitleResource(store: SpaceStore, source: SpaceCanvasNode) {
  if (!canCreateSubtitleResource(source) || source.data.kind !== "video" || !source.data.sourceRevision)
    return;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  let phase = "build-data";
  let outputId: string | undefined;
  try {
    const transcript = source.data.mediaKnowledge!.transcript;
    const data: SpaceSubtitlePayload = {
      kind: "subtitle",
      title: i18n.global.t("spaces.creation.video.subtitles.resourceTitle"),
      sourceVideoNodeId: source.id,
      sourceVideoRevision: source.data.sourceRevision,
      language: transcript.language,
      durationMs: transcript.durationMs,
      cues: transcript.subtitleCues.map(({ startMs, endMs, text }) => ({ startMs, endMs, text })),
    };
    phase = "create-node";
    const [output] = await store.createPendingContentForSpace(
      spaceId,
      data,
      derivedNodePosition([source], useCanvasEditorSession().selection.value.nodePlacements),
    );
    if (!output) throw new Error("Subtitle output node was not created");
    outputId = output.id;
    beginPendingDerivation(output.id, [source.id]);
    store.requestReveal([output.id]);
    await yieldToCanvas();
    phase = "persist-derivation";
    await invoke("plugin:spaces|create_subtitle_resource", {
      request: {
        id: crypto.randomUUID(),
        spaceId,
        sourceNodeId: source.id,
        outputNodeId: output.id,
        sourceRevision: source.data.sourceRevision,
      },
    });
    phase = "refresh-space";
    await store.refreshActiveSpace();
    store.finalizePendingContent(spaceId, [output]);
    store.requestReveal([output.id], "ensure-visible");
    return output;
  } catch (error) {
    void logError(`[subtitle-extraction] phase=${phase} error=${String(error)}`);
    if (outputId) await store.discardPendingContent(spaceId, outputId);
    toast.error(`${i18n.global.t("spaces.creation.video.subtitles.resourceFailed")}: ${String(error)}`);
    return undefined;
  } finally {
    if (outputId) endPendingDerivation(outputId);
  }
}
