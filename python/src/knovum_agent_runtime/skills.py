import base64
import hashlib
from collections.abc import Callable, Sequence

from langchain_core.tools import BaseTool
from langgraph.runtime import Runtime

from knovum_agent_runtime.contracts.generated.control import AgentSkillFile


def skill_files(files: list[AgentSkillFile]) -> list[tuple[str, bytes]]:
    resources: list[tuple[str, bytes]] = []
    for file in files:
        content = (
            file.content.encode("utf-8")
            if file.encoding.value == "utf8"
            else base64.b64decode(file.content, validate=True)
        )
        if hashlib.sha256(content).hexdigest() != file.digest:
            raise ValueError("agent-runtime-skill-content-digest-mismatch")
        resources.append((file.path, content))
    return resources


def skill_tool_resolver(
    bindings: dict[str, list[str]], tools: Sequence[BaseTool]
) -> Callable[[str, Runtime[None]], list[BaseTool]]:
    by_name = {tool.name: tool for tool in tools}

    def resolve(name: str, _runtime: Runtime[None]) -> list[BaseTool]:
        return [by_name[wire_name] for wire_name in bindings[name] if wire_name in by_name]

    return resolve
