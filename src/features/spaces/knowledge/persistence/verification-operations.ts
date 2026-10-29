import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { AnswerVerification } from "../../contracts/generated/AnswerVerification";
import type { ProposalReceipt } from "../../contracts/generated/ProposalReceipt";
import type { StatementSupport } from "../../contracts/generated/StatementSupport";

export function loadAnswerVerification(messageId: string): Promise<AnswerVerification | null> {
  return invokeCommand("plugin:spaces|load_answer_verification", { messageId });
}

export function verifyAnswer(messageId: string): Promise<void> {
  return invokeCommand("plugin:spaces|verify_answer", { messageId });
}

export function correctStatementSupport(
  messageId: string,
  statement: number,
  support: StatementSupport,
): Promise<void> {
  return invokeCommand("plugin:spaces|correct_statement_support", { messageId, statement, support });
}

export function recordAnswerClaims(
  messageId: string,
  statements: readonly number[],
): Promise<ProposalReceipt> {
  return invokeCommand("plugin:spaces|record_answer_claims", { messageId, statements });
}
