import type { EditorEvents } from "@tiptap/core";

/**
 * Whether an editor `update` changed the document. Tiptap also emits `update` from `setEditable`,
 * carrying an empty transaction, so making an editor read-only or editable again is not an edit.
 */
export function isDocumentEdit({ transaction, appendedTransactions }: EditorEvents["update"]) {
  return transaction.docChanged || appendedTransactions.some((appended) => appended.docChanged);
}
