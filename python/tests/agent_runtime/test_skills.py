from __future__ import annotations

import base64
import hashlib
import json

import pytest
from deepagents.backends import StoreBackend
from langchain_core.tools import tool
from langgraph.store.memory import InMemoryStore

from knovum_agent_runtime.contracts.generated.control import AgentSkillFile
from knovum_agent_runtime.skills import skill_files, skill_tool_resolver


def resource(path: str, content: bytes, binary: bool) -> AgentSkillFile:
    return AgentSkillFile.model_validate_json(
        json.dumps(
            {
                "path": path,
                "encoding": "base64" if binary else "utf8",
                "content": base64.b64encode(content).decode() if binary else content.decode(),
                "digest": hashlib.sha256(content).hexdigest(),
            }
        )
    )


def test_reviewed_skill_text_and_binary_survive_the_sdk_store_without_transcoding() -> None:
    files = [
        resource("/skills/example/SKILL.md", "资料与说明".encode(), False),
        resource("/skills/example/reference.bin", bytes(range(256)), True),
    ]
    backend = StoreBackend(namespace=lambda _: ("task", "skills"), store=InMemoryStore())
    assert all(upload.error is None for upload in backend.upload_files(skill_files(files)))
    downloads = backend.download_files([file.path for file in files])
    assert downloads[0].content == "资料与说明".encode()
    assert downloads[1].content == bytes(range(256))


def test_corrupt_binary_or_digest_is_rejected_before_the_skill_is_exposed() -> None:
    selected = resource("/skills/example/reference.bin", b"reviewed", True)
    with pytest.raises(ValueError, match="digest-mismatch"):
        skill_files([selected.model_copy(update={"digest": "0" * 64})])
    with pytest.raises(ValueError):
        skill_files([selected.model_copy(update={"content": "invalid*"})])


def test_skill_aliases_cannot_restore_write_tools_removed_from_a_readonly_graph() -> None:
    @tool
    def read_selected() -> str:
        """Read only the selected resource."""
        return "read"

    resolver = skill_tool_resolver(
        {"service/read": ["read_selected"], "service/write": ["write_selected"]}, [read_selected]
    )
    assert resolver("service/read", None) == [read_selected]
    assert resolver("service/write", None) == []
    with pytest.raises(KeyError):
        resolver("service/unselected", None)
