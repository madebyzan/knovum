import assert from "node:assert/strict";
import { test } from "node:test";

import { outputContract } from "../image-tensors.mjs";

const sample = { height: 2, width: 2 };
const rgb = { data: new Uint8Array([10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120]) };
const mask = { data: new Uint8Array([0, 255, 0, 255]) };
const inputs = { sample, rgb, mask };
const tensor = (data, dtype = "u1", shape = [1, 1, 2, 2]) => ({ data, dtype, shape });
test("repair protects every unselected channel while permitting selected changes", () => {
  const bytes = rgb.data.slice();
  bytes[1] = 255;
  outputContract({ capability: "repair" }, inputs, tensor(bytes, "u1", [1, 3, 2, 2]));
  bytes[4] = 255;
  assert.throws(
    () => outputContract({ capability: "repair" }, inputs, tensor(bytes, "u1", [1, 3, 2, 2])),
    /unselected/u,
  );
});
test("selection rejects nonbinary outputs and wrong layouts", () => {
  outputContract({ capability: "selection" }, inputs, tensor(new Uint8Array([0, 255, 0, 255])));
  assert.throws(
    () => outputContract({ capability: "selection" }, inputs, tensor(new Uint8Array([0, 1, 0, 255]))),
    /binary/u,
  );
  assert.throws(
    () => outputContract({ capability: "selection" }, inputs, tensor(new Uint8Array(4), "u1", [2, 2])),
    /shape/u,
  );
});
test("depth requires a varying field within the normalized range", () => {
  outputContract({ capability: "depth" }, inputs, tensor(new Float32Array([0, 0.25, 0.75, 1]), "f4"));
  for (const data of [
    [0, 0, 0, 0],
    [-1, 0, 0, 1],
    [0, 0, 0, 2],
  ])
    assert.throws(
      () => outputContract({ capability: "depth" }, inputs, tensor(new Float32Array(data), "f4")),
      /normalized/u,
    );
});
