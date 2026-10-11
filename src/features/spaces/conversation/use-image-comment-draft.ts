import { watch, type DeepReadonly, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import {
  useImageCommentSessionStore,
  type ImageCommentContext,
} from "../comments/image-comment-session-store";
import { useSpaceStore } from "../session/space-store";

function percent(value: number) {
  return `${Math.round(value * 1000) / 10}%`;
}

function coordinates(comment: DeepReadonly<ImageCommentContext>) {
  const shape = comment.shape;
  if (shape?.kind === "region") {
    return `x=${percent(shape.x)}, y=${percent(shape.y)}, width=${percent(shape.width)}, height=${percent(shape.height)}`;
  }
  if (shape?.kind === "stroke") {
    const xs = shape.points.filter((_, index) => index % 2 === 0);
    const ys = shape.points.filter((_, index) => index % 2 === 1);
    const left = Math.min(...xs);
    const top = Math.min(...ys);
    return `x=${percent(left)}, y=${percent(top)}, width=${percent(Math.max(...xs) - left)}, height=${percent(Math.max(...ys) - top)}`;
  }
  return `x=${percent(comment.x)}, y=${percent(comment.y)}`;
}

export function useImageCommentDraft(spaceId: Ref<string | undefined>, prompt: Ref<string>) {
  const { t } = useI18n();
  const store = useSpaceStore();
  const comments = useImageCommentSessionStore();
  watch(
    () => comments.conversationDraft,
    (draft) => {
      if (!draft || !spaceId.value || !store.nodes.some(({ id }) => id === draft.nodeId)) return;
      const shape = draft.comment.shape?.kind;
      const kind = shape === "stroke" ? "draw" : (shape ?? "point");
      const context = t("spaces.media.imageEditor.comments.conversationContext", {
        kind: t(`spaces.media.imageEditor.comments.${kind}`),
        coordinates: coordinates(draft.comment),
        text: draft.comment.text,
      });
      prompt.value = [prompt.value.trim(), context].filter(Boolean).join("\n\n");
      comments.takeConversationDraft();
    },
    { immediate: true },
  );
}
