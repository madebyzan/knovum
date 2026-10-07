import { z } from "zod";

const boundedIdentifier = z.string().trim().min(1).max(240);
const sha256Identifier = z.string().regex(/^[a-f0-9]{64}$/u, "Expected a lowercase SHA-256 identifier");
const timestamp = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
const revision = z.number().int().min(1).max(0xffff_ffff);

export const mediaSourceRevisionSchema = z.string().trim().min(1).max(128);

const spaceSubtitleCueSchema = z
  .object({
    startMs: timestamp,
    endMs: timestamp,
    text: z.string().trim().min(1).max(16_000),
  })
  .strict();

const transcriptUnitSchema = z
  .object({
    id: sha256Identifier,
    text: z.string().min(1).max(16_000),
    startCharacter: timestamp,
    endCharacter: timestamp,
    startMs: timestamp.nullable(),
    endMs: timestamp.nullable(),
    score: z.number().min(0).max(1).nullable(),
    unalignedReason: z.string().trim().min(1).max(120).nullable(),
    unitKind: z.enum(["word", "character", "segment"]),
  })
  .strict();

const spaceTranscriptSegmentSchema = z
  .object({
    id: sha256Identifier,
    revision,
    startMs: timestamp,
    endMs: timestamp,
    startSample: timestamp,
    endSample: timestamp,
    speakerId: boundedIdentifier.nullable(),
    text: z
      .string()
      .min(1)
      .max(16_000)
      .refine((value) => value.trim().length > 0),
    units: z.array(transcriptUnitSchema).min(1).max(16_000),
  })
  .strict();

const speakerTurnSchema = z.object({ startMs: timestamp, endMs: timestamp }).strict();
const speakerTrackSchema = z
  .object({
    speakerId: boundedIdentifier,
    turns: z.array(speakerTurnSchema).min(1).max(100_000),
  })
  .strict();

const transcriptProvenanceSchema = z
  .object({
    engineId: boundedIdentifier,
    modelSpecId: boundedIdentifier,
    modelRevision: boundedIdentifier,
    timing: z.enum(["engine_words", "segment_only"]),
  })
  .strict();

export const spaceMediaTranscriptSchema = z
  .object({
    id: sha256Identifier,
    sourceRevision: mediaSourceRevisionSchema,
    revision,
    language: z.string().trim().min(1).max(64),
    durationMs: timestamp,
    text: z.string().max(4 * 1024 * 1024),
    segments: z.array(spaceTranscriptSegmentSchema).max(100_000),
    subtitleCues: z.array(spaceSubtitleCueSchema).max(100_000),
    provenance: transcriptProvenanceSchema,
    alignment: z
      .object({
        engineId: boundedIdentifier,
        modelSpecId: boundedIdentifier,
        modelRevision: boundedIdentifier,
        alignedAt: timestamp,
      })
      .strict()
      .optional(),
    diarization: z
      .object({
        engineId: boundedIdentifier,
        modelSpecId: boundedIdentifier,
        modelRevision: boundedIdentifier,
        diarizedAt: timestamp,
      })
      .strict()
      .optional(),
    speakerTracks: z.array(speakerTrackSchema).max(1_024),
  })
  .strict();

export type SpaceTranscriptSegment = z.infer<typeof spaceTranscriptSegmentSchema>;
export type SpaceMediaTranscript = z.infer<typeof spaceMediaTranscriptSchema>;
