from __future__ import annotations

import json
from typing import Any

from knovum_agent_runtime.contracts.generated.models import AgentModelBinding, AgentModelPurpose, ChatModelProtocol
from knovum_agent_runtime.model_binding import OUTPUT_LIMIT_PARAMETER, create_model


def binding(protocol: str, model: str, **limits: int) -> AgentModelBinding:
    value: dict[str, Any] = {
        "protocol": protocol,
        "model": model,
        "baseUrl": "https://provider.test/v1",
        "apiVersion": None,
        "apiKey": "test-only",
        "temperature": 0.0,
        **limits,
    }
    return AgentModelBinding.model_validate_json(json.dumps(value))


def test_every_protocol_names_its_output_limit() -> None:
    assert set(OUTPUT_LIMIT_PARAMETER) == set(ChatModelProtocol)


def test_published_limits_are_the_model_profile_as_advertised() -> None:
    model = create_model(
        binding("open_ai_chat", "qwen-plus", maxInputTokens=1_000_000, maxOutputTokens=32_768), AgentModelPurpose.main
    )
    # The harness reserves output itself; the profile carries the advertised input limit untouched.
    assert model.profile == {"max_input_tokens": 1_000_000, "max_output_tokens": 32_768}
    assert model.max_tokens == 32_768


def test_unknown_limits_leave_each_client_its_own_profile_and_default() -> None:
    unknown = create_model(binding("open_ai_chat", "unlisted-model"), AgentModelPurpose.main)
    assert unknown.profile is None
    assert unknown.max_tokens is None
    bundled = create_model(binding("open_ai_chat", "gpt-4o"), AgentModelPurpose.main)
    assert bundled.profile is not None and bundled.profile["max_input_tokens"] > 0
    anthropic = create_model(binding("anthropic", "unlisted-model"), AgentModelPurpose.main)
    assert isinstance(anthropic.max_tokens, int) and anthropic.max_tokens > 0
