import { readFileSync, statSync } from "node:fs";

import { parse } from "npyjs";

import { EvidenceError, hashedFile } from "./evidence.mjs";

export function tensor(reference, base) {
  const path = hashedFile(reference, base);
  if (statSync(path).size > 256 * 1024 ** 2) throw new EvidenceError("Tensor exceeds its byte budget");
  const bytes = readFileSync(path);
  const value = parse(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength), {
    convertFloat16: false,
  });
  if (
    value.fortranOrder ||
    !["u1", "f4"].includes(value.dtype) ||
    value.data.some((v) => !Number.isFinite(v))
  )
    throw new EvidenceError("Tensor requires finite contiguous numeric values");
  return value;
}
export function shape(value, expected, dtype) {
  if (
    value.dtype !== dtype ||
    value.shape.join(",") !== expected.join(",") ||
    value.data.length !== expected.reduce((a, b) => a * b, 1)
  )
    throw new EvidenceError("Image tensor shape or dtype differs from its contract");
}
export function outputContract(profile, { sample, rgb, mask }, output) {
  const { height: h, width: w } = sample,
    capability = profile.capability;
  const [expected, dtype] = {
    upscale: [[1, 3, h * 4, w * 4], "u1"],
    repair: [[1, 3, h, w], "u1"],
    selection: [[1, 1, h, w], "u1"],
    depth: [[1, 1, h, w], "f4"],
  }[capability];
  shape(output, expected, dtype);
  if (
    capability === "repair" &&
    output.data.some((value, i) => mask.data[i % (h * w)] === 0 && value !== rgb.data[i])
  )
    throw new EvidenceError("Image repair changed unselected pixels");
  if (capability === "selection" && output.data.some((v) => v !== 0 && v !== 255))
    throw new EvidenceError("Image selection requires a binary mask");
  if (capability === "depth") {
    let min = Infinity,
      max = -Infinity;
    for (const value of output.data) {
      min = Math.min(min, value);
      max = Math.max(max, value);
    }
    if (min === max || min < 0 || max > 1)
      throw new EvidenceError("Relative depth requires a nonconstant normalized field");
  }
}
