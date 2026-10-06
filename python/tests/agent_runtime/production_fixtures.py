from __future__ import annotations

from typing import Any

from agent_runtime.sdk_fixtures import ScriptedModel

from knovum_agent_runtime.budget import ModelBudget
from knovum_agent_runtime.transport import HostRejected, wire_value


class LedgerHost:
    """Protocol peer recording the host's admission and checkpoint acknowledgements."""

    def __init__(self) -> None:
        self.calls: list[tuple[str, Any]] = []
        self.states: dict[str, str] = {}
        self.scopes: dict[str, str] = {}
        self.notifications: list[tuple[str, Any]] = []

    async def request(self, method: str, params: Any) -> Any:
        value = wire_value(params)
        self.calls.append((method, value))
        if method == "agentRuntime.admitModel":
            if any(state in {"unknown", "usage_unknown"} for state in self.states.values()):
                raise HostRejected(-32000, "agent-runtime-model-admission-denied")
            self.states[value["requestId"]] = "requesting"
            self.scopes[value["requestId"]] = value["graphNamespace"].rpartition("|")[0]
        elif method == "agentRuntime.completeModel":
            self.states[value["requestId"]] = "received" if value["inputTokens"] is not None else "usage_unknown"
        elif method == "agentRuntime.failModel":
            self.states[value["requestId"]] = "unknown"
        elif method == "agentRuntime.commitProjection":
            for proof in value["modelCheckpoints"]:
                for request_id in proof["requestIds"]:
                    if self.states.get(request_id) == "received" and self.scopes[request_id] == proof["graphNamespace"]:
                        self.states[request_id] = "committed"
        elif method == "agentRuntime.requireRecovery" and any(state != "committed" for state in self.states.values()):
            raise HostRejected(-32000, "model-outcome-unknown")
        return None

    async def notify(self, method: str, params: Any) -> None:
        self.notifications.append((method, wire_value(params)))


def bind_script(monkeypatch: Any, scripts: dict[str, list[Any]]) -> dict[str, ScriptedModel]:
    models: dict[str, ScriptedModel] = {}

    def create_model(binding: Any, purpose: Any) -> ScriptedModel:
        model = ScriptedModel(
            model=binding.model,
            api_key=binding.api_key,
            use_responses_api=False,
            max_retries=0,
            script=scripts[purpose.value],
            callbacks=[ModelBudget(purpose, binding.max_output_tokens)],
            tags=[f"knovum:model:{purpose.value}"],
            profile={
                "tool_calling": True,
                "max_input_tokens": binding.max_input_tokens,
                "max_output_tokens": binding.max_output_tokens,
            },
        )
        models[purpose.value] = model
        return model

    monkeypatch.setattr("knovum_agent_runtime.composition.create_model", create_model)
    return models
