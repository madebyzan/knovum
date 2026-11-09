import type { ProviderKind } from "../../../../shared/contracts/generated/ProviderKind";
import anthropic from "./assets/provider-logos/anthropic.svg?url";
import baichuan from "./assets/provider-logos/baichuan.svg?url";
import baidu from "./assets/provider-logos/baidu.svg?url";
import cerebras from "./assets/provider-logos/cerebras.svg?url";
import cohere from "./assets/provider-logos/cohere.svg?url";
import deepseek from "./assets/provider-logos/deepseek.svg?url";
import doubao from "./assets/provider-logos/doubao.svg?url";
import fireworks from "./assets/provider-logos/fireworks.svg?url";
import gemini from "./assets/provider-logos/gemini.svg?url";
import grok from "./assets/provider-logos/grok.svg?url";
import groq from "./assets/provider-logos/groq.svg?url";
import hunyuan from "./assets/provider-logos/hunyuan.svg?url";
import kling from "./assets/provider-logos/kling.svg?url";
import lmstudio from "./assets/provider-logos/lmstudio.svg?url";
import minimax from "./assets/provider-logos/minimax.svg?url";
import mistral from "./assets/provider-logos/mistral.svg?url";
import moonshot from "./assets/provider-logos/moonshot.svg?url";
import ollama from "./assets/provider-logos/ollama.svg?url";
import openai from "./assets/provider-logos/openai.svg?url";
import openrouter from "./assets/provider-logos/openrouter.svg?url";
import perplexity from "./assets/provider-logos/perplexity.svg?url";
import qwen from "./assets/provider-logos/qwen.svg?url";
import sensenova from "./assets/provider-logos/sensenova.svg?url";
import siliconflow from "./assets/provider-logos/siliconflow.svg?url";
import spark from "./assets/provider-logos/spark.svg?url";
import stepfun from "./assets/provider-logos/stepfun.svg?url";
import together from "./assets/provider-logos/together.svg?url";
import zhipu from "./assets/provider-logos/zhipu.svg?url";

type ProviderLogo = { src: string; monochrome?: true } | null;

export const LOGO_BY_PROVIDER: Record<ProviderKind, ProviderLogo> = {
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
  perplexity: { src: perplexity, monochrome: true },
  cohere: { src: cohere, monochrome: true },
  baidu: { src: baidu, monochrome: true },
  hunyuan: { src: hunyuan, monochrome: true },
  baichuan: { src: baichuan, monochrome: true },
  sensenova: { src: sensenova, monochrome: true },
  spark: { src: spark, monochrome: true },
  cerebras: { src: cerebras, monochrome: true },
  fireworks: { src: fireworks, monochrome: true },
  lmstudio: { src: lmstudio, monochrome: true },
};
