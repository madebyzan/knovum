// Generated from Rust contracts. Do not edit.
export const usagePurposes = ["assistant","editor","knowledge","knowledge_indexing","translation","visual_analysis"] as const;
export const providerKinds = ["openai","openai-compatible","openrouter","anthropic","gemini","qwen","deepseek","zhipu","siliconflow","kling","grok","groq","mistral","together","stepfun","moonshot","doubao","minimax","ollama"] as const;
export const executionCapabilities = ["text-generation","vision-understanding","image-generation","video-generation","embedding"] as const;
export const routingPurposes = ["writing","knowledge","visual-understanding","translation","image-generation","video-generation"] as const;
export const agentKinds = ["codex","claude_code","gemini_cli","github_copilot","open_code","qwen_code","qoder_cli","kimi_cli","auggie_cli","cline","cursor","antigravity"] as const;
export const routingCapabilities = {"writing":"text-generation","knowledge":"text-generation","visual-understanding":"vision-understanding","translation":"text-generation","image-generation":"image-generation","video-generation":"video-generation"} as const;
