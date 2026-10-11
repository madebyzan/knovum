import { EvidenceError, sameIds, unique } from "./evidence.mjs";
const bilingual = (samples) => {
  unique(samples.map((s) => s.id));
  sameIds([...new Set(samples.map((s) => s.language))], ["zh-CN", "en-US"]);
};
const imageCoverage = [
  "hair",
  "product_edges",
  "transparent",
  "text",
  "anime",
  "repeated_texture",
  "large_selection",
  "thin",
  "exif",
];

export function validateCorpus(corpus) {
  const embedding = corpus.embedding;
  unique(embedding.documents.map((s) => s.id));
  unique(embedding.queries.map((s) => s.id));
  sameIds([...new Set(embedding.documents.map((d) => d.kind))], ["document", "note", "code", "transcript"]);
  if (
    embedding.queries.some((q) =>
      q.relevantDocumentIds.some((id) => !embedding.documents.some((d) => d.id === id)),
    )
  )
    throw new EvidenceError("Unknown relevant document");
  for (const section of [corpus.asr, corpus.alignment, corpus.tts]) bilingual(section.samples);
  const samples = corpus.asr.samples,
    tags = new Set(samples.flatMap((s) => s.coverageTags));
  if (
    [
      "mixed_language",
      "proper_noun",
      "numbers",
      "silence",
      "noise",
      "speaker_overlap",
      "long_recording",
    ].some((t) => !tags.has(t)) ||
    new Set(samples.map((s) => s.sampleRateHz)).size < 2 ||
    !samples.some((s) => s.durationMs >= 600000)
  )
    throw new EvidenceError("ASR corpus lacks required coverage");
  for (const sample of corpus.alignment.samples) {
    unique(sample.units.map((u) => u.id));
    if (sample.units.some((u) => u.endMs < u.startMs)) throw new EvidenceError("Invalid annotation timeline");
  }
  if (
    corpus.alignment.thresholds.maximumP95BoundaryErrorMs <
    corpus.alignment.thresholds.maximumMedianBoundaryErrorMs
  )
    throw new EvidenceError("Inconsistent alignment thresholds");
}
export function validateImageCorpus(corpus) {
  unique(corpus.samples.map((s) => s.id));
  unique(corpus.profiles.map((p) => p.id));
  sameIds([...new Set(corpus.samples.flatMap((s) => s.coverage))], imageCoverage);
  for (const profile of corpus.profiles) {
    unique(profile.sampleIds);
    if (profile.sampleIds.some((id) => !corpus.samples.some((s) => s.id === id)))
      throw new EvidenceError("Unknown image corpus sample");
    sameIds(
      [...new Set(corpus.samples.filter((s) => profile.sampleIds.includes(s.id)).flatMap((s) => s.coverage))],
      imageCoverage,
    );
  }
}
