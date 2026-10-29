import type { AnswerStatement } from "../../contracts/generated/AnswerStatement";
import type { AnswerVerification } from "../../contracts/generated/AnswerVerification";
import type { StatementSupport } from "../../contracts/generated/StatementSupport";
import type { StatementVerdict } from "../../contracts/generated/StatementVerdict";

/** One statement as the person sees it: their correction wins over the model's verdict. */
export interface StatementView {
  statement: AnswerStatement;
  verdict: StatementVerdict | undefined;
  support: StatementSupport;
  corrected: boolean;
}

interface VerificationTally {
  checked: number;
  supported: number;
  partial: number;
  unsupported: number;
  uncited: number;
}

const GAPS: ReadonlySet<StatementSupport> = new Set(["partial", "unsupported", "uncited"]);

export function statementViews(verification: AnswerVerification): StatementView[] {
  const verdicts = new Map(verification.verdicts.map((verdict) => [verdict.statement, verdict]));
  const corrections = new Map(verification.corrections.map((entry) => [entry.statement, entry.support]));
  return verification.statements.flatMap((statement) => {
    const verdict = verdicts.get(statement.id);
    const correction = corrections.get(statement.id);
    const support = correction ?? verdict?.support;
    return support ? [{ statement, verdict, support, corrected: correction !== undefined }] : [];
  });
}

/** Counts only claims about the sources; inference and framing are not checked. */
export function tally(views: readonly StatementView[]): VerificationTally {
  const count = (support: StatementSupport) => views.filter((view) => view.support === support).length;
  const checked = views.filter((view) => view.support !== "not-checkable").length;
  return {
    checked,
    supported: count("supported"),
    partial: count("partial"),
    unsupported: count("unsupported"),
    uncited: count("uncited"),
  };
}

/** A statement the sources do not fully state: what the person should look at. */
export function isGap(view: StatementView): boolean {
  return GAPS.has(view.support);
}

/** Factual statements can be recorded as Claims; the sources decide whether as accepted or draft. */
export function isRecordable(view: StatementView): boolean {
  return view.support !== "not-checkable";
}
