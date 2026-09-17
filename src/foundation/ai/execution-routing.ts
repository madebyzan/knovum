import type { AiModelCapability, AiRoutingPurpose } from "./ai-models";

export interface ExecutionRoutingPolicy {
  modelCapability: AiModelCapability;
}

const routingPolicies: Record<AiRoutingPurpose, ExecutionRoutingPolicy> = {
  writing: {
    modelCapability: "text-generation",
  },
  knowledge: {
    modelCapability: "text-generation",
  },
  "visual-understanding": {
    modelCapability: "vision-understanding",
  },
  translation: {
    modelCapability: "text-generation",
  },
  "image-generation": {
    modelCapability: "image-generation",
  },
  "video-generation": {
    modelCapability: "video-generation",
  },
};

export const routingPolicy = (purpose: AiRoutingPurpose): ExecutionRoutingPolicy => routingPolicies[purpose];
