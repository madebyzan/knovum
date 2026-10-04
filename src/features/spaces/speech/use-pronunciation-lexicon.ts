import { computed, ref, watch } from "vue";

export interface PronunciationEntry {
  id: string;
  word: string;
  pronunciation: string;
}

export function usePronunciationLexicon(
  revision: () => Readonly<{ id: string; pronunciationLexicon: string }>,
) {
  const entries = ref<PronunciationEntry[]>([]);

  watch(
    () => revision().id,
    () => {
      const parsed = JSON.parse(revision().pronunciationLexicon) as Record<string, string>;
      entries.value = Object.entries(parsed).map(([word, pronunciation]) => ({
        id: crypto.randomUUID(),
        word,
        pronunciation,
      }));
    },
    { immediate: true },
  );

  const valid = computed(() => {
    if (entries.value.length > 1_000) return false;
    const words = new Set<string>();
    for (const entry of entries.value) {
      const word = normalize(entry.word);
      const pronunciation = normalize(entry.pronunciation);
      if (
        !word ||
        [...word].length > 80 ||
        !pronunciation ||
        [...pronunciation].length > 160 ||
        words.has(word)
      )
        return false;
      words.add(word);
    }
    return true;
  });

  function value() {
    if (!valid.value) throw new Error("Pronunciation lexicon entries are invalid");
    return Object.fromEntries(
      entries.value.map((entry) => [normalize(entry.word), normalize(entry.pronunciation)]),
    );
  }

  return { entries, valid, value };
}

function normalize(value: string) {
  return value.trim().split(/\s+/u).join(" ");
}
