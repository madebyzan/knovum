import { z } from "zod";

/** Words taken from elsewhere in the app, with the canvas items they came from. */
export interface CanvasPassage {
  text: string;
  sourceNodeIds: readonly string[];
}

// A type only Knovum writes, so a drop from another app is never read as one.
const PASSAGE_TYPE = "application/x-knovum-passage";
const passageSchema = z
  .object({ text: z.string().min(1), sourceNodeIds: z.array(z.string().min(1)).max(256) })
  .strict();

/** Drags `passage`; every other drop target still receives its plain text. */
export function writePassageTransfer(transfer: DataTransfer, passage: CanvasPassage) {
  transfer.setData("text/plain", passage.text);
  transfer.setData(PASSAGE_TYPE, JSON.stringify(passage));
  transfer.effectAllowed = "copy";
}

export function readPassageTransfer(transfer: DataTransfer): CanvasPassage | undefined {
  const raw = transfer.getData(PASSAGE_TYPE);
  if (!raw) return undefined;
  return passageSchema.parse(JSON.parse(raw));
}
