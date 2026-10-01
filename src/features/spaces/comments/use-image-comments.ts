import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { errorMessage } from "../../../foundation/errors/error-format";
import { useContextDock } from "../canvas/use-context-dock";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import { useImageCommentSession } from "./image-comment-session";
import { useImageCommentGesture, type CommentMark } from "./use-image-comment-gesture";

type ImageComment = NonNullable<SpaceImagePayload["comments"]>[number];
export interface ImageCommentCanvas {
  id: string;
  data: SpaceImagePayload;
  imageRect: { left: number; top: number; width: number; height: number };
  interactive: boolean;
  frameRotation: number;
}

function positionStyle(position: { x: number; y: number }) {
  return { left: `${position.x * 100}%`, top: `${position.y * 100}%` };
}

export function useImageComments(props: ImageCommentCanvas) {
  const { t } = useI18n();
  const store = useSpaceStore();
  const dock = useContextDock();
  const session = useImageCommentSession();
  const adding = computed(() => props.interactive && session.activeNodeId === props.id);
  const hintLabel = computed(() =>
    t(
      session.mode === "point"
        ? "spaces.media.imageEditor.comments.hint"
        : session.mode === "draw"
          ? "spaces.media.imageEditor.comments.draw"
          : "spaces.media.imageEditor.comments.region",
    ),
  );
  const draft = shallowRef<CommentMark>();
  const text = shallowRef("");
  const selectedId = shallowRef<string>();
  const busy = shallowRef(false);
  const comments = computed(() => props.data.comments ?? []);
  const areaStyle = computed(() => ({
    left: `${props.imageRect.left}px`,
    top: `${props.imageRect.top}px`,
    width: `${props.imageRect.width}px`,
    height: `${props.imageRect.height}px`,
  }));
  function complete(mark: CommentMark) {
    if (!adding.value || comments.value.length >= 200) return;
    draft.value = mark;
    selectedId.value = "draft";
    text.value = "";
  }
  function placeCenter() {
    if (session.mode === "point") complete({ x: 0.5, y: 0.5 });
  }
  const gesture = useImageCommentGesture(complete, () => props.frameRotation);
  watch(() => session.mode, gesture.cancel);
  watch(
    adding,
    (value) => {
      if (value) return;
      gesture.cancel();
      draft.value = undefined;
      selectedId.value = undefined;
      text.value = "";
    },
    { flush: "sync" },
  );
  async function persist(next: ImageComment[]) {
    const node = store.nodes.find(({ id }) => id === props.id);
    if (!node || node.data.kind !== "image") return;
    busy.value = true;
    try {
      await store.updateNodeContents([
        { id: props.id, before: node.data, after: { ...node.data, comments: next } },
      ]);
      draft.value = undefined;
      selectedId.value = undefined;
      text.value = "";
    } catch (error) {
      toast.error(errorMessage(error));
    } finally {
      busy.value = false;
    }
  }
  function saveComment() {
    const value = text.value.trim();
    const mark = draft.value;
    if (!mark || !value || busy.value) return;
    const comment = { id: crypto.randomUUID(), ...mark, text: value, createdAt: Date.now() };
    void persist([...comments.value, comment]);
  }
  function addToConversation() {
    const value = text.value.trim();
    const mark = draft.value;
    if (!mark || !value || busy.value) return;
    session.addToConversation(props.id, { ...mark, text: value });
    session.cancel();
    dock.openAsk("selection", { nodeIds: [props.id] });
  }
  function remove(comment: ImageComment) {
    if (!busy.value) void persist(comments.value.filter(({ id }) => id !== comment.id));
  }
  function discuss(comment: ImageComment) {
    session.addToConversation(props.id, comment);
    session.cancel();
    dock.openAsk("selection", { nodeIds: [props.id] });
  }
  function close(open: boolean, id: string) {
    selectedId.value = open ? id : undefined;
    if (!open && id === "draft") draft.value = undefined;
  }
  function cancelDraft() {
    draft.value = undefined;
    selectedId.value = undefined;
  }
  return {
    t,
    adding,
    hintLabel,
    mode: computed(() => session.mode),
    preview: gesture.preview,
    startGesture: gesture.start,
    moveGesture: gesture.move,
    finishGesture: gesture.finish,
    cancelGesture: gesture.cancel,
    placeCenter,
    draft,
    text,
    selectedId,
    busy,
    comments,
    areaStyle,
    positionStyle,
    saveComment,
    addToConversation,
    remove,
    discuss,
    close,
    cancelDraft,
  };
}
