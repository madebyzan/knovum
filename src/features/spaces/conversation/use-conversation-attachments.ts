import { basename } from "pathe";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { spaceCanvasOverviewSource } from "../assets/node-preview-source";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { currentSelectionRoots } from "../canvas/selection/selection-roots";
import type { SubmitAssistantTaskRequest } from "../contracts/generated/SubmitAssistantTaskRequest";
import { createStructureIndex } from "../model/structure/structure-index";
import { contentNodeIdsWithin } from "../model/structure/structure-queries";
import { useSpaceStore } from "../session/space-store";
import { chooseAssistantUploads, inspectAssistantUploads } from "./conversation-uploads";
import { previewConversationAttachment } from "./preview-conversation-attachment";
import { isAssistantImagePath, useConversationImagePreviews } from "./use-conversation-image-previews";

type Attachment = NonNullable<SubmitAssistantTaskRequest["attachments"]>[number];

function attachmentErrorMessage(error: unknown, t: ReturnType<typeof useI18n>["t"]) {
  const code = decodeAppError(error)?.code;
  const known: Readonly<Record<string, string>> = {
    assistant_context_limit: "spaces.assistant.attachments.limit",
    assistant_upload_size: "spaces.assistant.uploads.sizeError",
    assistant_upload_unsupported: "spaces.assistant.uploads.typeError",
    assistant_upload_invalid: "spaces.assistant.uploads.invalidError",
    assistant_upload_unauthorized: "spaces.assistant.uploads.permissionError",
  };
  const key = code ? known[code] : undefined;
  return key ? t(key) : getErrorMessage(error);
}

export function useConversationAttachments() {
  const { t } = useI18n();
  const canvas = useCanvasEditorSession();
  const store = useSpaceStore();
  const attachments = shallowRef<Attachment[]>([]);
  const imagePreview = useConversationImagePreviews(attachments);
  const pickingFiles = shallowRef(false);
  const attachmentError = shallowRef<string>();
  const selectedSourceNodeIds = computed(() => {
    if (!canvas.selection.value.nodeIds.length && !canvas.selection.value.structureNodeIds.length)
      return new Set<string>();
    const index = createStructureIndex(store.editSnapshot.structure);
    const roots = currentSelectionRoots(
      canvas.selection.value.nodeIds,
      canvas.selection.value.structureNodeIds,
      index,
    );
    return new Set(
      contentNodeIdsWithin(
        roots.map(({ id }) => id),
        index,
      ),
    );
  });
  const visibleAttachments = computed(() =>
    attachments.value.map((attachment) => {
      if (attachment.kind === "file")
        return {
          key: `file:${attachment.path}`,
          kind: attachment.kind,
          title: basename(attachment.path),
          isImage: isAssistantImagePath(attachment.path),
          previewSource: imagePreview(attachment),
        };
      const node = store.nodes.find(({ id }) => id === attachment.nodeId);
      return {
        key: `node:${attachment.nodeId}`,
        kind: attachment.kind,
        title: node?.data.title ?? attachment.nodeId,
        isImage: node?.data.kind === "image",
        previewSource: node ? spaceCanvasOverviewSource(node.data) : "",
      };
    }),
  );
  const attachmentCandidates = computed(() => {
    const attachedIds = new Set(
      attachments.value.flatMap((item) => (item.kind === "node" ? [item.nodeId] : [])),
    );
    return store.nodes
      .filter((node) => Boolean(node.data.knowledgeRevision ?? node.data.sourceRevision))
      .map((node) => ({
        nodeId: node.id,
        title: node.data.title,
        previewSource: spaceCanvasOverviewSource(node.data),
        selected: selectedSourceNodeIds.value.has(node.id),
        attached: attachedIds.has(node.id),
      }));
  });

  function addSelectedAttachments() {
    addNodeAttachments([...selectedSourceNodeIds.value]);
  }

  function addNodeAttachments(nodeIds: readonly string[]) {
    const selected = new Set(nodeIds);
    const next = new Map(
      attachments.value.map((item) => [
        item.kind === "node" ? `node:${item.nodeId}` : `file:${item.path}`,
        item,
      ]),
    );
    for (const node of store.nodes) {
      if (!selected.has(node.id)) continue;
      const revision = node.data.knowledgeRevision ?? node.data.sourceRevision;
      if (revision)
        next.set(`node:${node.id}`, {
          kind: "node",
          nodeId: node.id,
          sourceRevision: revision,
          role: "source",
        });
    }
    if (next.size > 32) {
      attachmentError.value = t("spaces.assistant.attachments.limit");
      return;
    }
    attachmentError.value = undefined;
    attachments.value = [...next.values()];
  }

  async function chooseFiles(isCurrent: () => boolean, paths?: readonly string[]) {
    if (pickingFiles.value) return;
    pickingFiles.value = true;
    attachmentError.value = undefined;
    try {
      const files = await (paths ? inspectAssistantUploads(paths) : chooseAssistantUploads());
      if (!isCurrent()) return;
      const next = new Map(
        attachments.value.map((item) => [
          item.kind === "node" ? `node:${item.nodeId}` : `file:${item.path}`,
          item,
        ]),
      );
      for (const file of files)
        next.set(`file:${file.path}`, {
          kind: "file",
          path: file.path,
          sourceRevision: file.sourceRevision,
        });
      if (next.size > 32) {
        attachmentError.value = t("spaces.assistant.attachments.limit");
        return;
      }
      attachments.value = [...next.values()];
    } catch (error) {
      if (!isCurrent()) return;
      attachmentError.value = attachmentErrorMessage(error, t);
    } finally {
      pickingFiles.value = false;
    }
  }

  function removeAttachment(key: string) {
    attachments.value = attachments.value.filter(
      (item) => (item.kind === "node" ? `node:${item.nodeId}` : `file:${item.path}`) !== key,
    );
    attachmentError.value = undefined;
  }

  return {
    attachments,
    visibleAttachments,
    attachmentCandidates,
    attachmentError,
    pickingFiles,
    addSelectedAttachments,
    addNodeAttachments,
    chooseFiles,
    removeAttachment,
    previewAttachment: (key: string) => previewConversationAttachment(attachments.value, store, key),
  };
}
