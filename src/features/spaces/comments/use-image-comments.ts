import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

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
        ? "spaces.media.commentHint"
        : session.mode === "draw"
          ? "spaces.media.commentDraw"
          : "spaces.media.commentRegion",
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
  const gesture = useImageCommentGesture(complete);
  watch(() => session.mode, gesture.cancel);
  watch(adding, (value) => {
    if (!value) gesture.cancel();
  });
  async function persist(next: ImageComment[]) {
    const node = store.nodes.find(({ id }) => id === props.id);
    if (!node || node.data.kind !== "image") return false;
    busy.value = true;
    try {
      await store.updateManyData([
        { id: props.id, before: node.data, after: { ...node.data, comments: next } },
      ]);
      draft.value = undefined;
      selectedId.value = undefined;
      text.value = "";
      return true;
    } catch (error) {
      toast.error(String(error));
      return false;
    } finally {
      busy.value = false;
    }
  }
  async function saveDraft() {
    const value = text.value.trim();
    const mark = draft.value;
    if (!mark || !value || busy.value) return;
    const saved = await persist([
      ...comments.value,
      { id: crypto.randomUUID(), ...mark, text: value, createdAt: Date.now() },
    ]);
    return saved ? value : undefined;
  }
  function send() {
    void saveDraft();
  }
  async function addToConversation() {
    const value = await saveDraft();
    if (!value) return;
    session.addToConversation(props.id, value);
    dock.openAsk("selection", { nodeIds: [props.id] });
  }
  function remove(comment: ImageComment) {
    if (!busy.value) void persist(comments.value.filter(({ id }) => id !== comment.id));
  }
  function discuss(comment: ImageComment) {
    session.addToConversation(props.id, comment.text);
    selectedId.value = undefined;
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
    setMode: session.setMode,
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
    send,
    addToConversation,
    remove,
    discuss,
    close,
    cancelDraft,
  };
}
