import type { ProviderKind } from "../../../shared/contracts/generated/ProviderKind";
import anthropic from "../assets/provider-logos/anthropic.svg?url";
import deepseek from "../assets/provider-logos/deepseek.svg?url";
import doubao from "../assets/provider-logos/doubao.svg?url";
import gemini from "../assets/provider-logos/gemini.svg?url";
import grok from "../assets/provider-logos/grok.svg?url";
import groq from "../assets/provider-logos/groq.svg?url";
import kling from "../assets/provider-logos/kling.svg?url";
import minimax from "../assets/provider-logos/minimax.svg?url";
import mistral from "../assets/provider-logos/mistral.svg?url";
import moonshot from "../assets/provider-logos/moonshot.svg?url";
import ollama from "../assets/provider-logos/ollama.svg?url";
import openai from "../assets/provider-logos/openai.svg?url";
import openrouter from "../assets/provider-logos/openrouter.svg?url";
import qwen from "../assets/provider-logos/qwen.svg?url";
import siliconflow from "../assets/provider-logos/siliconflow.svg?url";
import stepfun from "../assets/provider-logos/stepfun.svg?url";
import together from "../assets/provider-logos/together.svg?url";
import zhipu from "../assets/provider-logos/zhipu.svg?url";

type ProviderLogo = { src: string; monochrome?: true } | null;

export const providerLogos: Record<ProviderKind, ProviderLogo> = {
  openai: { src: openai, monochrome: true },
  "openai-compatible": null,
  openrouter: { src: openrouter },
  anthropic: { src: anthropic, monochrome: true },
  gemini: { src: gemini },
  qwen: { src: qwen },
  deepseek: { src: deepseek },
  zhipu: { src: zhipu },
  siliconflow: { src: siliconflow },
  kling: { src: kling },
  grok: { src: grok, monochrome: true },
  groq: { src: groq, monochrome: true },
  mistral: { src: mistral },
  together: { src: together },
  stepfun: { src: stepfun },
  moonshot: { src: moonshot, monochrome: true },
  doubao: { src: doubao },
  minimax: { src: minimax },
  ollama: { src: ollama, monochrome: true },
};
