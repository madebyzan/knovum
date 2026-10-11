import json
from pathlib import Path

import pytest

from policy import validate_license_review


def review(tmp_path: Path, status: str, approval=None) -> Path:
    (tmp_path / "packaging").mkdir()
    (tmp_path / "packaging/license-review.json").write_text(
        json.dumps(
            {
                "schemaVersion": 1,
                "reviewStatus": status,
                "targets": {"macos-x64": approval},
            }
        )
    )
    return tmp_path


def test_qa_does_not_claim_license_approval(tmp_path: Path):
    project = review(tmp_path, "pending")
    validate_license_review(project, tmp_path, "macos-x64", False)
    with pytest.raises(ValueError, match="unresolved"):
        validate_license_review(project, tmp_path, "macos-x64", True)


def test_approval_is_bound_to_current_dependency_evidence(tmp_path: Path, monkeypatch):
    monkeypatch.setattr("policy.review_inputs", lambda _project, _root: "current")
    project = review(tmp_path, "approved", {"reviewInputSha256": "obsolete"})
    with pytest.raises(ValueError, match="current target dependencies"):
        validate_license_review(project, tmp_path, "macos-x64", True)


def test_matching_approval_still_requires_accountability(tmp_path: Path, monkeypatch):
    monkeypatch.setattr("policy.review_inputs", lambda _project, _root: "current")
    project = review(
        tmp_path, "approved", {"reviewInputSha256": "current", "evidenceLocation": "https://example.com/review"}
    )
    with pytest.raises(ValueError, match="reviewer"):
        validate_license_review(project, tmp_path, "macos-x64", True)
