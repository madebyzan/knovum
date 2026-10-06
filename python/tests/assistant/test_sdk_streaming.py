from __future__ import annotations

import asyncio
import json
from pathlib import Path

import httpx
import pytest
from assistant.sdk_fixtures import AdmissionRecorder
from deepagents import create_deep_agent
from deepagents.profiles import GeneralPurposeSubagentProfile, HarnessProfile, register_harness_profile
from langchain_core.messages import HumanMessage
from langchain_openai import ChatOpenAI
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver


@pytest.mark.parametrize("denied", [False, True])
def test_official_streaming_client_obeys_admission_and_commits_answer(tmp_path: Path, denied: bool) -> None:
    async def run() -> None:
        requests: list[dict] = []

        def endpoint(request: httpx.Request) -> httpx.Response:
            body = json.loads(request.content)
            requests.append(body)
            assert request.url.path == "/v1/chat/completions"
            assert request.headers["authorization"] == "Bearer a0-test-credential"
            chunks = [
                {
                    "choices": [
                        {"index": 0, "delta": {"role": "assistant", "content": "complete "}, "finish_reason": None}
                    ]
                },
                {"choices": [{"index": 0, "delta": {"content": "answer"}, "finish_reason": None}]},
                {
                    "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
                    "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
                },
            ]
            payload = "".join(
                "data: "
                + json.dumps(
                    {"id": "stream-a0", "object": "chat.completion.chunk", "created": 0, "model": "knovum-a0", **chunk}
                )
                + "\n\n"
                for chunk in chunks
            )
            return httpx.Response(
                200, headers={"content-type": "text/event-stream"}, content=payload + "data: [DONE]\n\n"
            )

        register_harness_profile(
            "openai:knovum-a0",
            HarnessProfile(general_purpose_subagent=GeneralPurposeSubagentProfile(enabled=False)),
        )
        recorder = AdmissionRecorder(deny=denied)
        config = {"configurable": {"thread_id": "stream-thread"}}
        async with httpx.AsyncClient(transport=httpx.MockTransport(endpoint)) as client:
            model = ChatOpenAI(
                model="knovum-a0",
                api_key="a0-test-credential",
                base_url="https://a0.example/v1",
                use_responses_api=False,
                max_retries=0,
                stream_usage=True,
                callbacks=[recorder],
                http_async_client=client,
            )
            async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
                graph = create_deep_agent(model=model, checkpointer=saver)
                events = []

                async def stream() -> None:
                    async for event in graph.astream(
                        {"messages": [HumanMessage(content="hello", id="stream-input")]},
                        config,
                        stream_mode=["messages", "updates"],
                        durability="sync",
                    ):
                        events.append(event)

                if denied:
                    with pytest.raises(PermissionError, match="admission denied"):
                        await stream()
                    assert requests == []
                    assert recorder.completions == []
                else:
                    await stream()
                    assert len(requests) == 1
                    assert requests[0]["stream"] is True
                    state = await graph.aget_state(config)
                    message = state.values["messages"][-1]
                    assert message.content == "complete answer"
                    assert message.id
                    assert message.usage_metadata["total_tokens"] == 15
                    streamed = [payload for mode, payload in events if mode == "messages"]
                    assert any(chunk.content == "complete " for chunk, metadata in streamed)
                    assert all(metadata["langgraph_node"] == "model" for chunk, metadata in streamed)
                    assert len(recorder.starts) == len(recorder.completions) == 1

    asyncio.run(run())
