import type { Ref, ShallowRef } from "vue";

import type { AgentExtensionSelection } from "../../../shared/contracts/generated/AgentExtensionSelection";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import type { VisibleConversationAttachment } from "./conversation-pending-message";
import type { LiveTranscript } from "./conversation-transcript";

export interface ConversationSubmissionState {
  activeRun: ShallowRef<RunObservation | undefined>;
  activeRunId: ShallowRef<string | undefined>;
  activeConversationId: ShallowRef<string>;
  activeSpaceId: ShallowRef<string | undefined>;
  taskState: ShallowRef<string | undefined>;
  loading: ShallowRef<boolean>;
  error: ShallowRef<string | undefined>;
  live: ShallowRef<LiveTranscript | undefined>;
  activityRevision: ShallowRef<number>;
}
export interface ConversationSubmissionInput {
  extensions: Ref<AgentExtensionSelection>;
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  conversation: Ref<Conversation | undefined>;
  prompt: Ref<string>;
  target: Ref<TaskTarget | undefined>;
  attachments: Ref<ConversationAttachment[]>;
  visibleAttachments: Ref<readonly VisibleConversationAttachment[]>;
  hasUnresolvedFileReferences: Ref<boolean>;
  messages: ShallowRef<ConversationMessage[]>;
  ensureSpacePersisted: () => Promise<void>;
  clearDraft: (request: SubmitConversationTaskRequest) => Promise<void>;
  restoreDraft: (request: SubmitConversationTaskRequest) => Promise<void>;
}
