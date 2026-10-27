export interface ConversationAttachmentPresentation {
  key: string;
  kind: "node" | "file";
  title: string;
  isImage: boolean;
  previewSource: string;
}

export interface ConversationAttachmentCandidatePresentation {
  nodeId: string;
  title: string;
  previewSource: string;
  selected: boolean;
  attached: boolean;
}

/**
 * The file type a document attachment is shown with, read from its name ("deck.pptx" → "PPTX");
 * undefined when the name carries none, as a canvas node's title usually does.
 */
export function attachmentFileType(title: string): string | undefined {
  return /\.((?=[a-z0-9]{1,5}$)[a-z0-9]*[a-z][a-z0-9]*)$/i.exec(title.trim())?.[1]?.toUpperCase();
}
