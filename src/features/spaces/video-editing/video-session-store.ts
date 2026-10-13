import { cloneDeep, isEqual } from "es-toolkit";
import { defineStore } from "pinia";
import { markRaw, shallowRef, type Ref } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { videoEditProjectSchema } from "../model/nodes/schema/video-edit";
import type { SpaceEditorTab } from "../workspace/workbench-tab";
import type { VideoEditProject } from "./project";
import type { VideoSelection } from "./selection";
import { useVideoEditHistory } from "./use-video-edit-history";

/** A stored project only opens when it matches the current contract; older shapes are rejected. */
function readProject(data: SpaceVideoPayload) {
  if (!data.videoEditProject) return { project: undefined, obsolete: false };
  const parsed = videoEditProjectSchema.safeParse(data.videoEditProject);
  return parsed.success
    ? { project: cloneDeep(parsed.data) as VideoEditProject, obsolete: false }
    : { project: undefined, obsolete: true };
}

export const useVideoSessionStore = defineStore("spaces-video-sessions", () => {
  const sessions = shallowRef<ReadonlyMap<string, VideoSession>>(new Map());

  function open(tab: SpaceEditorTab, node: SpaceCanvasNode<SpaceVideoPayload>) {
    const stored = readProject(node.data);
    const project = shallowRef<VideoEditProject | undefined>(stored.project);
    const history = useVideoEditHistory(project);
    history.reset(stored.project);
    const session = markRaw({
      tab,
      node: shallowRef(node),
      project,
      history,
      saving: shallowRef(false),
      rendering: shallowRef(false),
      error: shallowRef<"missing" | "conflict">(),
      obsolete: shallowRef(stored.obsolete),
      selection: shallowRef<VideoSelection>(),
      timelineMs: shallowRef(0),
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
    if (!changed) return;
    const stored = readProject(node.data);
    session.project.value = stored.project;
    session.obsolete.value = stored.obsolete;
    session.history.reset(stored.project);
    session.timelineMs.value = 0;
    session.selection.value = undefined;
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
  /** A stored project from an older contract; the editor offers an explicit reset. */
  obsolete: Ref<boolean>;
  selection: Ref<VideoSelection | undefined>;
  timelineMs: Ref<number>;
}
