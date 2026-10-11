import { z } from "zod";

export const text = z.string().regex(/\S/u);
export const hash = z.string().regex(/^[a-f0-9]{64}$/u);
export const timestamp = z.iso.datetime({ offset: true });
export const target = z.enum(["macos-arm64", "macos-x64", "windows-x64"]);
export const file = z.strictObject({ path: text, sha256: hash });
export const configuration = z.strictObject({ id: text, parameters: z.record(z.string(), z.json()) });
export const model = z.strictObject({ specId: text, identityDigest: hash });
export const device = z.strictObject({
  manufacturer: text,
  model: text,
  cpu: text,
  physicalCores: z.int().positive(),
  memoryBytes: z.int().positive(),
  os: text,
  osVersion: text,
  powerMode: text,
  accelerator: text,
  driver: text.nullable(),
});
export const source = z.strictObject({
  kind: z.enum(["bundled_worker", "official_python_reference", "product_baseline"]),
  method: text,
  workerDigest: hash.nullable().optional(),
  runtimeDigest: hash,
  modelSpecId: text,
  configId: text,
  invocations: z
    .array(
      z.strictObject({
        requestId: text,
        attemptEpoch: z.int().nonnegative(),
        startedAt: timestamp,
        finishedAt: timestamp,
      }),
    )
    .min(1),
});
