// Generated from Rust contracts. Do not edit.
export const usagePurposes = ["builtin_agent","editor","knowledge","knowledge_indexing","translation","visual_analysis"] as const;
export const providerKinds = ["openai","openai-compatible","openrouter","anthropic","gemini","qwen","deepseek","zhipu","siliconflow","kling","grok","groq","mistral","together","stepfun","moonshot","doubao","minimax","ollama"] as const;
export const executionCapabilities = ["text-generation","vision-understanding","image-generation","video-generation","embedding"] as const;
export const routingPurposes = ["writing","knowledge","visual-understanding","translation","image-generation","video-generation"] as const;
export const chatModelProtocols = ["open_ai_chat","open_ai_responses","anthropic","gemini","ollama","deep_seek","groq"] as const;
export const externalAgentKinds = ["codex","claude_code","gemini_cli","github_copilot","open_code","qwen_code","qoder_cli","kimi_cli","auggie_cli","cline","cursor","antigravity"] as const;
export const preferenceThemes = ["system","light","dark"] as const;
export const preferenceDensities = ["compact","comfortable"] as const;
export const preferenceTextSizes = ["small","default","large"] as const;
export const preferenceLocales = ["system","en","zh-CN"] as const;
export const routingCapabilities = {"writing":"text-generation","knowledge":"text-generation","visual-understanding":"vision-understanding","translation":"text-generation","image-generation":"image-generation","video-generation":"video-generation"} as const;
export const preferencesSchemaVersion = 1 as const;
export const workbenchWidthLimits = {"sidebar":[200,320],"rightSidebar":[300,480]} as const;
export const preferenceDefaults = {"theme":"system","density":"compact","textSize":"default","locale":"system","backgroundTaskNotifications":true,"workbench":{"sidebarVisible":true,"sidebarWidth":224,"rightSidebarVisible":false,"rightSidebarWidth":380}} as const;
