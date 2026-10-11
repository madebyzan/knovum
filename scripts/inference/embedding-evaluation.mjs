import { EvidenceError, execution, sameIds } from "./evidence.mjs";

export function cosine(left, right) {
  if (
    !left.length ||
    left.length !== right.length ||
    [...left, ...right].some((item) => !Number.isFinite(item))
  )
    throw new EvidenceError("Vectors require matching finite dimensions");
  let dot = 0,
    a = 0,
    b = 0;
  for (let i = 0; i < left.length; i++) {
    dot += left[i] * right[i];
    a += left[i] ** 2;
    b += right[i] ** 2;
  }
  const value = dot / Math.sqrt(a) / Math.sqrt(b);
  if (!Number.isFinite(value)) throw new EvidenceError("Vectors require finite nonzero norms");
  return Math.max(-1, Math.min(1, value));
}
const mean = (items) => items.reduce((a, b) => a + b, 0) / items.length;
function retrieval(values, queries) {
  const recalls = [],
    ndcgs = [];
  for (const query of queries) {
    const ranked = Object.entries(values.documents)
      .map(([id, vector]) => ({ id, score: cosine(values.queries[query.id], vector) }))
      .toSorted((a, b) => b.score - a.score || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))
      .slice(0, 10);
    const relevant = new Set(query.relevantDocumentIds);
    if (!relevant.size) throw new EvidenceError("Retrieval query has no relevant documents");
    recalls.push(ranked.filter((item) => relevant.has(item.id)).length / relevant.size);
    const dcg = ranked.reduce((sum, item, i) => sum + (relevant.has(item.id) ? 1 / Math.log2(i + 2) : 0), 0);
    const ideal = Array.from({ length: Math.min(10, relevant.size) }, (_, i) => 1 / Math.log2(i + 2)).reduce(
      (a, b) => a + b,
      0,
    );
    ndcgs.push(dcg / ideal);
  }
  return [mean(recalls), mean(ndcgs)];
}
export function evaluateEmbedding(corpus, section, base, provenance) {
  const reference = execution(section.referenceOutput, base, "embedding", {
    provenance,
    kind: "official_python_reference",
    profile: corpus.referenceProfile,
  });
  const candidate = execution(section.candidateOutput, base, "embedding", { provenance });
  const queryIds = corpus.queries.map((item) => item.id),
    documentIds = corpus.documents.map((item) => item.id);
  for (const values of [reference, candidate]) {
    sameIds(Object.keys(values.queries), queryIds);
    sameIds(Object.keys(values.documents), documentIds);
    const dimensions = new Set(
      [...Object.values(values.queries), ...Object.values(values.documents)].map((v) => v.length),
    );
    if (dimensions.size !== 1 || dimensions.has(0)) throw new EvidenceError("Embedding dimensions differ");
  }
  const paired = [
    ...queryIds.map((id) => cosine(reference.queries[id], candidate.queries[id])),
    ...documentIds.map((id) => cosine(reference.documents[id], candidate.documents[id])),
  ];
  const [referenceRecallAt10, referenceNdcgAt10] = retrieval(reference, corpus.queries),
    [candidateRecallAt10, candidateNdcgAt10] = retrieval(candidate, corpus.queries);
  const minimumPairedCosine = Math.min(...paired),
    recallAt10Drop = referenceRecallAt10 - candidateRecallAt10,
    ndcgAt10Drop = referenceNdcgAt10 - candidateNdcgAt10;
  const passed =
    (section.comparisonMode !== "cpu_float32" || minimumPairedCosine >= 0.9999) &&
    recallAt10Drop <= 0.01 &&
    ndcgAt10Drop <= 0.01;
  return {
    status: passed ? "passed" : "failed",
    comparisonMode: section.comparisonMode,
    minimumPairedCosine,
    cosineThreshold: section.comparisonMode === "cpu_float32" ? 0.9999 : null,
    referenceRecallAt10,
    candidateRecallAt10,
    recallAt10Drop,
    referenceNdcgAt10,
    candidateNdcgAt10,
    ndcgAt10Drop,
    maximumRetrievalDrop: 0.01,
  };
}
