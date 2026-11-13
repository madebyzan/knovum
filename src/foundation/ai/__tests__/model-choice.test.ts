import { describe, expect, it } from "vitest";

import { decideModelChoice, preferredChoiceId, type ModelChoiceOption } from "../model-choice";

function option(id: string, overrides: Partial<ModelChoiceOption> = {}): ModelChoiceOption {
  return { id, name: id, runnable: true, preferred: false, ...overrides };
}

describe("model choice policy", () => {
  it("runs the only executor even when it is not the Settings preference", () => {
    expect(decideModelChoice([option("downloaded")])).toEqual({ kind: "run", id: "downloaded" });
  });

  it("asks when several can run and preselects the Settings preference", () => {
    const decision = decideModelChoice([option("a"), option("b", { preferred: true }), option("c")]);
    expect(decision).toMatchObject({ kind: "choose", selectedId: "b" });
  });

  it("preselects the first runnable executor when the preference cannot run", () => {
    const decision = decideModelChoice([
      option("missing", { runnable: false, preferred: true }),
      option("a"),
      option("b"),
    ]);
    expect(decision).toMatchObject({ kind: "choose", selectedId: "a" });
    expect(decision.kind === "choose" && decision.options.map(({ id }) => id)).toEqual(["a", "b"]);
  });

  it("offers nothing when no executor can run yet", () => {
    expect(decideModelChoice([option("a", { runnable: false })])).toEqual({ kind: "none" });
    expect(decideModelChoice([])).toEqual({ kind: "none" });
    expect(preferredChoiceId([])).toBeUndefined();
  });
});
