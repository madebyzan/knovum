import { describe, expect, it } from "vitest";

import { promptArguments, resourceParameters } from "../catalog-input";

describe("extension catalog input", () => {
  it("validates actual RFC6570 parameter kinds and exact prompt argument values", () => {
    expect(resourceParameters('{"id":"a/b","labels":["one"],"pairs":{"key":"value"}}')).toEqual({
      id: "a/b",
      labels: ["one"],
      pairs: { key: "value" },
    });
    expect(() => resourceParameters('{"id":12}')).toThrow(/Invalid input/);
    expect(() => promptArguments('{"subject":12}')).toThrow(/expected string/);
  });
});
