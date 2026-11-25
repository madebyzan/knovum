const CJK_QUERY_PATTERN = /[\u3040-\u30ff\u3400-\u9fff\uac00-\ud7af]/;

/**
 * Short CJK queries (often 1-2 ideographs) score lower under `fast-fuzzy`'s Damerau-Levenshtein
 * similarity against multi-word labels than Latin substrings do. Relaxing the threshold for CJK
 * queries keeps Chinese/Japanese/Korean search responsive without loosening Latin matching.
 */
export function fuzzySearchThreshold(query: string, latinThreshold = 0.6) {
  return CJK_QUERY_PATTERN.test(query) ? Math.min(latinThreshold, 0.45) : latinThreshold;
}
