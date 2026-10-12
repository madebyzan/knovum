import { cloneDeep, isEqual } from "es-toolkit";
import { defineStore } from "pinia";
import { markRaw, shallowRef, type Ref } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import type { SpaceEditorTab } from "../workspace/work-items";
import type { VideoEditProject } from "./project";
import { useVideoEditHistory } from "./use-video-edit-history";

export const useVideoSessionStore = defineStore("spaces-video-sessions", () => {
  const sessions = shallowRef<ReadonlyMap<string, VideoSession>>(new Map());
  function open(tab: SpaceEditorTab, node: SpaceCanvasNode<SpaceVideoPayload>) {
    const project = shallowRef<VideoEditProject | undefined>(
      node.data.videoEditProject ? cloneDeep(node.data.videoEditProject) : undefined,
    );
    const history = useVideoEditHistory(project);
    history.reset(node.data.videoEditProject);
    const session = markRaw({
      tab,
      node: shallowRef(node),
      project,
      history,
      saving: shallowRef(false),
      rendering: shallowRef(false),
      error: shallowRef<"missing" | "conflict">(),
      selectedClipId: shallowRef<string>(),
      selectedTextId: shallowRef<string>(),
      selectedAudioId: shallowRef<string>(),
      timelineMs: shallowRef(0),
      timelineZoom: shallowRef(1),
    });
    sessions.value = new Map(sessions.value).set(tab.id, session);
  }
  function synchronize(tabId: string, node: SpaceCanvasNode | undefined) {
    const session = sessions.value.get(tabId);
    if (!session || session.saving.value) return;
    if (!node || node.data.kind !== "video") {
      session.error.value = "missing";
      return;
    }
    const changed =
      node.data.sourceRevision !== session.node.value.data.sourceRevision ||
      !isEqual(node.data.videoEditProject, session.node.value.data.videoEditProject);
    if (changed && session.history.dirty.value) {
      session.error.value = "conflict";
      return;
    }
    session.node.value = node as SpaceCanvasNode<SpaceVideoPayload>;
    session.error.value = undefined;
    if (changed) {
      session.project.value = node.data.videoEditProject ? cloneDeep(node.data.videoEditProject) : undefined;
      session.history.reset(node.data.videoEditProject);
      session.timelineMs.value = 0;
      session.selectedClipId.value = undefined;
      session.selectedTextId.value = undefined;
      session.selectedAudioId.value = undefined;
    }
  }
  function release(tabId: string) {
    const next = new Map(sessions.value);
    next.delete(tabId);
    sessions.value = next;
  }
  return { sessions, open, synchronize, release };
});

export interface VideoSession {
  tab: SpaceEditorTab;
  node: Ref<SpaceCanvasNode<SpaceVideoPayload>>;
  project: Ref<VideoEditProject | undefined>;
  history: ReturnType<typeof useVideoEditHistory>;
  saving: Ref<boolean>;
  rendering: Ref<boolean>;
  error: Ref<"missing" | "conflict" | undefined>;
  selectedClipId: Ref<string | undefined>;
  selectedTextId: Ref<string | undefined>;
  selectedAudioId: Ref<string | undefined>;
  timelineMs: Ref<number>;
  timelineZoom: Ref<number>;
}
