import type { z } from "zod";

import type { spaceNodePayloadSchema } from "./schema/node-variants";

export type SpaceNodePayload = z.infer<typeof spaceNodePayloadSchema>;
export type SpaceNodeKind = SpaceNodePayload["kind"];

export type SpaceNotePayload = Extract<SpaceNodePayload, { kind: "note" }>;
export type SpaceTextPayload = Extract<SpaceNodePayload, { kind: "text" }>;
export type SpaceMarkdownPayload = Extract<SpaceNodePayload, { kind: "markdown" }>;
export type SpaceWebsitePayload = Extract<SpaceNodePayload, { kind: "website" }>;
export type SpaceImagePayload = Extract<SpaceNodePayload, { kind: "image" }>;
export type SpaceVideoPayload = Extract<SpaceNodePayload, { kind: "video" }>;
export type SpaceAudioPayload = Extract<SpaceNodePayload, { kind: "audio" }>;
export type SpaceSubtitlePayload = Extract<SpaceNodePayload, { kind: "subtitle" }>;
export type SpaceFilePayload = Extract<SpaceNodePayload, { kind: "file" }>;
export type SpaceCodePayload = Extract<SpaceNodePayload, { kind: "code" }>;
export type SpaceCredentialPayload = Extract<SpaceNodePayload, { kind: "credential" }>;
export type SpaceKnowledgeObjectPayload = Extract<SpaceNodePayload, { kind: "object" }>;
export type SpacePortalPayload = Extract<SpaceNodePayload, { kind: "portal" }>;
export type SpaceResourceEntry = NonNullable<SpaceFilePayload["entries"]>[number];
