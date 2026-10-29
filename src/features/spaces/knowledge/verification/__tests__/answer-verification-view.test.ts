import { describe, expect, it } from "vitest";

import type { AnswerVerification } from "../../../contracts/generated/AnswerVerification";
import type { StatementSupport } from "../../../contracts/generated/StatementSupport";
import { isGap, isRecordable, statementViews, tally } from "../answer-verification-view";

function verification(supports: StatementSupport[], corrections: AnswerVerification["corrections"] = []) {
  return {
    messageId: "message",
    state: "completed",
    skip: null,
    truncated: false,
    error: null,
    completedAt: 1,
    corrections,
    statements: supports.map((_, index) => ({
      id: index + 1,
      text: `Statement ${index + 1}`,
      start: 0,
      end: 1,
      citations: [1],
    })),
    verdicts: supports.map((support, index) => ({
      statement: index + 1,
      role: support === "not-checkable" ? "framing" : "factual",
      support,
      evidence: [],
      possibleSources: [],
      reason: "",
    })),
  } satisfies AnswerVerification;
}

describe("answer verification view", () => {
  it("counts claims about the sources and leaves framing out", () => {
    const views = statementViews(verification(["supported", "partial", "uncited", "not-checkable"]));
    expect(tally(views)).toEqual({ checked: 3, supported: 1, partial: 1, unsupported: 0, uncited: 1 });
    expect(views.filter(isGap).map((view) => view.statement.id)).toEqual([2, 3]);
    expect(views.filter(isRecordable).map((view) => view.statement.id)).toEqual([1, 2, 3]);
  });

  it("shows the person's correction instead of the model's verdict", () => {
    const views = statementViews(verification(["unsupported"], [{ statement: 1, support: "supported" }]));
    expect(views[0]).toMatchObject({ support: "supported", corrected: true });
    expect(tally(views).supported).toBe(1);
  });

  it("omits statements a failed or pending verification never judged", () => {
    const pending = { ...verification(["supported"]), verdicts: [] };
    expect(statementViews(pending)).toEqual([]);
  });
});
