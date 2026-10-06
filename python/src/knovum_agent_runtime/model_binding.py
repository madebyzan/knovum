from __future__ import annotations

from typing import Any

from langchain_anthropic import ChatAnthropic
from langchain_core.language_models import BaseChatModel
from langchain_deepseek import ChatDeepSeek
from langchain_google_genai import ChatGoogleGenerativeAI
from langchain_groq import ChatGroq
from langchain_ollama import ChatOllama
from langchain_openai import ChatOpenAI
from pydantic import SecretStr

from knovum_agent_runtime.budget import ModelBudget
from knovum_agent_runtime.contracts.generated.models import AgentModelBinding, AgentModelPurpose, ChatModelProtocol


def create_model(binding: AgentModelBinding, purpose: AgentModelPurpose) -> BaseChatModel:
    callbacks = [ModelBudget(purpose, binding.max_output_tokens)]
    parameters: dict[str, Any] = {
        "model": binding.model,
        "temperature": binding.temperature,
        "callbacks": callbacks,
        "profile": {
            "tool_calling": True,
            "max_input_tokens": binding.max_input_tokens,
            "max_output_tokens": binding.max_output_tokens,
        },
        "tags": [f"knovum:model:{purpose.value}"],
    }
    if binding.protocol != ChatModelProtocol.ollama:
        parameters["api_key"] = SecretStr(binding.api_key) if binding.api_key is not None else None
    match binding.protocol:
        case ChatModelProtocol.open_ai_chat | ChatModelProtocol.open_ai_responses:
            return ChatOpenAI(
                **parameters,
                base_url=binding.base_url,
                max_completion_tokens=binding.max_output_tokens,
                timeout=60,
                max_retries=0,
                stream_usage=True,
                use_responses_api=binding.protocol == ChatModelProtocol.open_ai_responses,
                use_previous_response_id=False,
                store=False if binding.protocol == ChatModelProtocol.open_ai_responses else None,
                include=["reasoning.encrypted_content"]
                if binding.protocol == ChatModelProtocol.open_ai_responses
                else None,
            )
        case ChatModelProtocol.anthropic:
            return ChatAnthropic(
                **parameters,
                base_url=binding.base_url,
                max_tokens_to_sample=binding.max_output_tokens,
                timeout=60,
                max_retries=0,
            )
        case ChatModelProtocol.gemini:
            return ChatGoogleGenerativeAI(
                **parameters,
                base_url=binding.base_url,
                api_version=binding.api_version,
                max_tokens=binding.max_output_tokens,
                timeout=60,
                # Google counts the initial request in attempts; zero enables defaults.
                max_retries=1,
                vertexai=False,
            )
        case ChatModelProtocol.ollama:
            return ChatOllama(
                **parameters,
                base_url=binding.base_url,
                num_predict=binding.max_output_tokens,
                client_kwargs={
                    "timeout": 60,
                    **({"headers": {"Authorization": f"Bearer {binding.api_key}"}} if binding.api_key else {}),
                },
            )
        case ChatModelProtocol.deep_seek:
            return ChatDeepSeek(
                **parameters,
                base_url=binding.base_url,
                max_tokens=binding.max_output_tokens,
                timeout=60,
                max_retries=0,
            )
        case ChatModelProtocol.groq:
            return ChatGroq(
                **parameters,
                base_url=binding.base_url,
                max_tokens=binding.max_output_tokens,
                timeout=60,
                max_retries=0,
            )
    raise ValueError("agent-runtime-model-protocol-unsupported")
