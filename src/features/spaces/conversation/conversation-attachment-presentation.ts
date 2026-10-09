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
