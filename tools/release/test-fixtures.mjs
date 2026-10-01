import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { artifactName, releaseTargets } from "./manifest.mjs";

const key = Buffer.concat([Buffer.from("Ed"), Buffer.alloc(40)]).toString("base64");
export const testPublicKey = Buffer.from(`untrusted comment: fixture\n${key}\n`).toString("base64");
const packet = Buffer.concat([Buffer.from("ED"), Buffer.alloc(72)]).toString("base64");
export const testSignature = Buffer.from(`untrusted comment: fixture\n${packet}\ntrusted comment: fixture\n${Buffer.alloc(64).toString("base64")}\n`).toString("base64");

export function releaseFixture(context) {
  const directory = mkdtempSync(join(tmpdir(), "knovum-release-test-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  for (const target of releaseTargets) {
    const name = artifactName("0.0.2", target);
    writeFileSync(join(directory, name), `test-only artifact ${target}`);
    writeFileSync(join(directory, `${name}.sig`), testSignature);
    if (target.startsWith("darwin-")) writeFileSync(join(directory, name.replace(".app.tar.gz", ".dmg")), "test DMG");
  }
  const notesFile = join(directory, "notes.md");
  writeFileSync(notesFile, "Fix updater configuration and release distribution.");
  return { directory, notesFile, repository: "owner/knovum-releases", version: "0.0.2", publicKey: testPublicKey };
}
