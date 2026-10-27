import type { ConversationToolOperation } from "../contracts/generated/ConversationToolOperation";

export interface ToolOperationPresentation {
  titleKey: string;
  descriptionKey?: string;
  /** What the user decides on: the command line itself, or the recorded call. */
  detail: string;
}

/**
 * How an operation waiting on the user reads. A command is shown as the command line it would run,
 * and the user learns it runs with their own permissions — Knovum only asks when it cannot confine
 * a command to the Agent's working directory. Any other operation shows its recorded call.
 */
export function toolOperationPresentation(operation: ConversationToolOperation): ToolOperationPresentation {
  const command = commandLine(operation);
  const approval = operation.state === "prepared";
  if (command === undefined) {
    const detail = JSON.stringify(
      { source: operation.source, context: operation.context, output: operation.output },
      null,
      2,
    );
    return approval
      ? { titleKey: "spaces.conversation.toolOperations.approval.title", detail }
      : {
          titleKey: "spaces.conversation.toolOperations.outcomeUnknown.title",
          descriptionKey: "spaces.conversation.toolOperations.outcomeUnknown.description",
          detail,
        };
  }
  return {
    titleKey: approval
      ? "spaces.conversation.toolOperations.command.approval"
      : "spaces.conversation.toolOperations.outcomeUnknown.title",
    descriptionKey: approval
      ? "spaces.conversation.toolOperations.command.unconfined"
      : "spaces.conversation.toolOperations.command.outcomeUnknown",
    detail: command,
  };
}

function commandLine(operation: ConversationToolOperation): string | undefined {
  if (operation.source.type !== "command") return undefined;
  const { context } = operation;
  const command =
    context !== null && typeof context === "object" && !Array.isArray(context) ? context.command : undefined;
  return typeof command === "string" ? command : undefined;
}
