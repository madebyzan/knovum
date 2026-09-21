import type { Ref } from "vue";

import { addImportedSpaceAsset } from "../assets/import";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { PersistedAudioSpaceAsset } from "./audio-recording";

export interface SpaceAudioRecordingTarget {
  spaceId: string;
  position: { x: number; y: number };
}

interface AudioRecordingActionContext {
  activeSpaceId: Ref<string | undefined>;
  target: Ref<SpaceAudioRecordingTarget | undefined>;
  nextInsertionPoint(): { x: number; y: number };
  ensureActiveSpacePersisted(): Promise<void>;
  createPendingContentForSpace(
    spaceId: string,
    data: SpaceNodePayload,
    position: { x: number; y: number },
  ): Promise<SpaceCanvasNode[]>;
  finalizePendingContent(spaceId: string, contentNodes: readonly SpaceCanvasNode[]): void;
}

export function createSpaceAudioRecordingActions(context: AudioRecordingActionContext) {
  async function beginAudioRecording() {
    if (context.target.value) return;
    await context.ensureActiveSpacePersisted();
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("Audio recording requires an active Space");
    context.target.value = { spaceId, position: context.nextInsertionPoint() };
  }

  function closeAudioRecording() {
    context.target.value = undefined;
  }

  async function saveAudioRecording(asset: PersistedAudioSpaceAsset, spaceId: string) {
    const target = context.target.value;
    if (!target || target.spaceId !== spaceId)
      throw new Error("Audio recording target changed before the recording was saved");
    await addImportedSpaceAsset(asset, target.position, async (data, position) => {
      const contentNodes = await context.createPendingContentForSpace(spaceId, data, position);
      context.finalizePendingContent(spaceId, contentNodes);
      return contentNodes;
    });
  }

  return { beginAudioRecording, closeAudioRecording, saveAudioRecording };
}
