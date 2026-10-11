from pathlib import Path

from policy import executable_options, runtime_distributions


def test_windows_keeps_stdio_and_declares_utf8():
    options = executable_options("win32")
    assert options["console"] is True
    assert "manifest" in options
    assert "<activeCodePage" in Path(options["manifest"]).read_text()
    assert "UTF-8" in Path(options["manifest"]).read_text()


def test_macos_requires_complete_onedir_instead_of_onefile():
    options = executable_options("darwin")
    assert options["console"] is True
    assert options["exclude_binaries"] is True
    assert "manifest" not in options


def test_metadata_closure_contains_providers_and_excludes_build_tools():
    names = runtime_distributions()
    assert {"deepagents", "langgraph", "langchain-openai", "truststore"} <= names
    assert not {"pyinstaller", "pytest", "ruff", "mypy", "setuptools"} & names


def test_runtime_metadata_honors_dependency_extras(monkeypatch):
    from types import SimpleNamespace

    import policy

    declarations = {"knovum-agent": ["sdk[transport]"], "sdk": ['transport; extra == "transport"'], "transport": []}
    monkeypatch.setattr(policy, "distribution", lambda name: SimpleNamespace(requires=declarations[name]))
    assert runtime_distributions() == {"knovum-agent", "sdk", "transport"}
