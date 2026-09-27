import { describe, expect, it } from "vitest";
import { reactive } from "vue";

import type { AiConnectionSettings } from "../../../foundation/ai/schema/models";
import { createAiConnectionDraft, finalizeAiConnectionDraft } from "../ai-connection-draft";

const connection: AiConnectionSettings = {
  id: "openai-main",
  name: "OpenAI",
  provider: "openai",
  baseUrl: "https://api.openai.com/v1",
  models: [
    {
      id: "gpt",
      name: "GPT",
      model: "gpt-5",
      capabilities: ["text-generation"],
    },
  ],
};

describe("AI connection drafts", () => {
  it("creates an editable value object from reactive settings", () => {
    const source = reactive(connection);
    const draft = createAiConnectionDraft(source);

    draft.models[0]!.name = "Changed";

    expect(source.models[0]!.name).toBe("GPT");
  });

  it("normalizes editable fields at the save boundary", () => {
    const draft = createAiConnectionDraft(connection);
    draft.name = "  Primary  ";
    draft.baseUrl = "  https://api.openai.com/v1  ";
    draft.models[0]!.model = "  gpt-5  ";

    expect(finalizeAiConnectionDraft(draft)).toMatchObject({
      name: "Primary",
      baseUrl: "https://api.openai.com/v1",
      models: [{ model: "gpt-5" }],
    });
  });
});
