import { flushPromises, mount } from "@vue/test-utils";
import { expect, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { createSpaceNode } from "../../canvas/node";
import { useDocumentSessionStore } from "../../document-editing/document-session-store";
import { createSpaceNotePayload } from "../../document-editing/tiptap-content";
import { useSpaceTiptapEditor } from "../../document-editing/use-space-tiptap-editor";

/** Opens one document tab of the Space "origin" and mounts its editor until the test finishes. */
export async function openDocument(kind: "note" | "markdown" = "note") {
  const tab = { id: "document-tab", kind: "document", nodeId: "node", spaceId: "origin" } as const;
  const node = createSpaceNode(
    kind === "note" ? createSpaceNotePayload("Original") : { kind: "markdown", title: "Original", text: "" },
    { x: 0, y: 0 },
    tab.nodeId,
  );
  const sessions = useDocumentSessionStore();
  sessions.open(tab, node);
  const active = shallowRef(true);
  let document!: ReturnType<typeof useSpaceTiptapEditor>;
  const wrapper = mount(
    {
      setup() {
        document = useSpaceTiptapEditor(tab, active, shallowRef(null));
        return () => null;
      },
    },
    {
      global: {
        plugins: [
          createI18n({
            legacy: false,
            locale: "en",
            missingWarn: false,
            fallbackWarn: false,
            messages: { en: { spaces: { editor: { placeholder: "Start writing…" } } } },
          }),
        ],
      },
    },
  );
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  await vi.waitFor(() => expect(document.editor.value?.isInitialized).toBe(true));
  return { tab, node, active, document, session: sessions.sessions.get(tab.id)! };
}
