from __future__ import annotations

from typing import Any

from deepagents import create_deep_agent
from deepagents.backends import CompositeBackend, FilesystemBackend, StoreBackend
from deepagents.middleware import FilesystemPermission
from deepagents.middleware.skills import SkillsMiddleware
from deepagents.middleware.subagents import SubAgent
from deepagents.middleware.summarization import create_summarization_middleware
from deepagents.profiles import GeneralPurposeSubagentProfile, HarnessProfile, register_harness_profile
from langchain.agents.middleware import AgentMiddleware, TodoListMiddleware
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from langgraph.graph.state import CompiledStateGraph
from langgraph.store.memory import InMemoryStore

from knovum_agent_runtime.contracts.generated.control import AgentExecute
from knovum_agent_runtime.contracts.generated.models import AgentModelPurpose
from knovum_agent_runtime.control import ControlMiddleware, ExecutionContext
from knovum_agent_runtime.host_tools import create_tool
from knovum_agent_runtime.model_binding import create_model
from knovum_agent_runtime.skills import skill_files, skill_tool_resolver

MODEL_PROVIDERS = {
    "open_ai_chat": "openai",
    "open_ai_responses": "openai",
    "anthropic": "anthropic",
    "gemini": "google_genai",
    "ollama": "ollama",
    "deep_seek": "deepseek",
    "groq": "groq",
}

PLANNING_PROMPT = (
    "## Planning\n\nFor work with several distinct steps, keep a short plan with `write_todos`: "
    "one item in_progress at a time, marked completed as soon as it is done. Skip it for simple requests."
)
PLANNING_TOOL = (
    "Replace the task plan. Each item has content and a status of pending, in_progress or completed. "
    "Use only for multi-step work."
)


def create_agent_graph(request: AgentExecute, saver: AsyncSqliteSaver) -> CompiledStateGraph[Any, Any, Any, Any]:
    provider = MODEL_PROVIDERS[request.model.protocol.value]
    register_harness_profile(
        f"{provider}:{request.model.model}",
        HarnessProfile(general_purpose_subagent=GeneralPurposeSubagentProfile(enabled=False)),
    )
    skills_backend = StoreBackend(namespace=lambda _: ("reviewed_skills", "v1"), store=InMemoryStore())
    # The conversation's working directory is the file tools' `/`; `virtual_mode` keeps every path
    # inside it. Knovum, not this backend, runs commands (`run_command`), so deepagents never
    # offers its own unrecorded `execute`. Offloaded tool results and history stay out of sight.
    backend = CompositeBackend(
        default=FilesystemBackend(root_dir=request.workspace, virtual_mode=True),
        routes={"/skills/": skills_backend},
        artifacts_root="/.agent",
    )
    uploaded = backend.upload_files(skill_files(request.skills))
    if any(item.error for item in uploaded):
        raise ValueError("agent-runtime-skills-unavailable")
    main = create_model(request.model, AgentModelPurpose.main)
    summarizer = create_model(request.model, AgentModelPurpose.context_summary)

    def execution_middleware(*, read_only: bool = False) -> list[AgentMiddleware[Any, Any, Any]]:
        tools = [create_tool(tool) for tool in request.tools if not read_only or tool.read_only]
        return [
            # Deep Agents derives trigger, retention and the input budget from the model profile;
            # only the summary model differs, so its calls are metered under their own purpose.
            create_summarization_middleware(summarizer, backend),
            SkillsMiddleware(
                backend=backend, sources=["/skills/"], tools=skill_tool_resolver(request.skill_tool_bindings, tools)
            ),
            ControlMiddleware(read_only=read_only),
        ]

    middleware = execution_middleware()
    middleware.append(TodoListMiddleware(system_prompt=PLANNING_PROMPT, tool_description=PLANNING_TOOL))
    reviewed_skills = FilesystemPermission(operations=["write"], paths=["/skills", "/skills/**"], mode="deny")
    read_only = FilesystemPermission(operations=["write"], paths=["/**"], mode="deny")
    subagents: list[SubAgent] = [
        {
            "name": "research",
            "description": "Read authorized sources and report evidence without publishing changes.",
            "system_prompt": "Use the read-only Knovum tools and return a concise report with evidence.",
            "model": create_model(request.model, AgentModelPurpose.research),
            "tools": [create_tool(tool) for tool in request.tools if tool.read_only],
            "permissions": [read_only],
            "middleware": execution_middleware(read_only=True),
        }
    ]
    return create_deep_agent(
        model=main,
        system_prompt=request.system_prompt,
        tools=[create_tool(tool) for tool in request.tools],
        middleware=middleware,
        subagents=subagents,
        backend=backend,
        skills=["/skills/"],
        permissions=[reviewed_skills],
        context_schema=ExecutionContext,
        checkpointer=saver,
    )
