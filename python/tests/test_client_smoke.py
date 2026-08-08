"""Generated-client smoke test.

Deterministic and network-free: monkeypatches the module-level httpx transport
so the client is exercised against a mocked discovery file + canned responses.
No Rust server is required, so this gate runs on every CI build.
"""

from __future__ import annotations

import json
from pathlib import Path

import httpx
import pytest

from steel_horizons_client import SteelHorizonsClient, read_discovery_file


def _write_discovery(tmp_path: Path) -> None:
    (tmp_path / "connection.json").write_text(
        json.dumps(
            {
                "protocol": "http",
                "host": "127.0.0.1",
                "port": 4880,
                "token": "secret-token",
                "pid": 123,
            }
        )
    )


def test_read_discovery_file(tmp_path: Path) -> None:
    """read_discovery_file parses the engine's connection.json."""
    _write_discovery(tmp_path)
    info = read_discovery_file(tmp_path)
    assert info["host"] == "127.0.0.1"
    assert info["port"] == 4880
    assert info["token"] == "secret-token"


def test_from_discovery_and_commands(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """from_discovery wires host/port/token; status/command hit the right
    endpoints with the bearer header."""
    _write_discovery(tmp_path)

    calls: list[tuple[str, str, dict]] = []

    def fake_get(url: str, **kwargs: object) -> httpx.Response:
        calls.append(("get", url, kwargs))
        return httpx.Response(
            200,
            json={"status": "ok", "protocol_version": "v1"},
            request=httpx.Request("GET", url),
        )

    def fake_post(url: str, **kwargs: object) -> httpx.Response:
        calls.append(("post", url, kwargs))
        return httpx.Response(
            200,
            json={"status": "applied", "id": "cmd-1"},
            request=httpx.Request("POST", url),
        )

    monkeypatch.setattr(httpx, "get", fake_get)
    monkeypatch.setattr(httpx, "post", fake_post)

    client = SteelHorizonsClient.from_discovery(tmp_path)
    assert client._base == "http://127.0.0.1:4880"
    assert client._headers["Authorization"] == "Bearer secret-token"

    # status() -> GET /api/v1/status with the bearer header.
    s = client.status()
    assert s["status"] == "ok"
    assert calls[0][0] == "get"
    assert "/api/v1/status" in calls[0][1]
    assert calls[0][2]["headers"]["Authorization"] == "Bearer secret-token"

    # command() -> POST /api/v1/command with an envelope and the bearer header.
    ack = client.command({"id": "cmd-1", "command": {"type": "pause"}})
    assert ack["status"] == "applied"
    assert calls[1][0] == "post"
    assert "/api/v1/command" in calls[1][1]
    assert calls[1][2]["headers"]["Authorization"] == "Bearer secret-token"
