import { expect, it } from "vitest";

import { workflowOutputNamesValid } from "../output-name-policy";

it.each([
  { names: [" Summary ", "Illustration"], valid: true },
  { names: ["   "], valid: false },
  { names: ["Same", " Same "], valid: false },
  { names: ["😀".repeat(120)], valid: true },
  { names: ["😀".repeat(121)], valid: false },
])("validates trimmed, unique output names by Unicode code point", ({ names, valid }) => {
  expect(workflowOutputNamesValid(names)).toBe(valid);
});
