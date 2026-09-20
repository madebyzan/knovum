import type { z } from "zod";

import type { spaceClipboardPayloadSchema } from "./payload-schema";
export type SpaceClipboardPayload = z.infer<typeof spaceClipboardPayloadSchema>;
export type SpaceClipboardNode = SpaceClipboardPayload["nodes"][number];
