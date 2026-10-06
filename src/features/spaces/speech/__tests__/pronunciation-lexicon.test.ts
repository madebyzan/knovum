import { expect, it } from "vitest";
import { effectScope, shallowRef } from "vue";

import { usePronunciationLexicon } from "../use-pronunciation-lexicon";

it("keeps revision entries, editable draft rows and saved normalized words independent", () => {
  const scope = effectScope();
  try {
    scope.run(() => {
      const revision = shallowRef({ id: "first", pronunciationLexicon: { Knovum: "Knowledge" } });
      const draft = usePronunciationLexicon(() => revision.value);
      expect(draft.value()).toEqual({ Knovum: "Knowledge" });
      draft.entries.value[0]!.pronunciation = "  New   sound  ";
      expect(draft.value()).toEqual({ Knovum: "New sound" });
      expect(revision.value.pronunciationLexicon).toEqual({ Knovum: "Knowledge" });
      draft.entries.value.push({ id: "duplicate", word: " Knovum ", pronunciation: "Other" });
      expect(draft.valid.value).toBe(false);
      expect(() => draft.value()).toThrow("invalid");
    });
  } finally {
    scope.stop();
  }
});
