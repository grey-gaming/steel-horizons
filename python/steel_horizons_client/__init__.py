"""
Steel Horizons REST API client — minimal Python wrapper.

Endpoints
---------
- GET  /api/v1/status          — server status
- GET  /api/v1/state            — full game state snapshot
- GET  /api/v1/content          — content catalog
- GET  /api/v1/state/{coll}     — collection query (ships, stations, bodies, research, build-orders)
- GET  /api/v1/state/{coll}/{id}— single entity lookup
- POST /api/v1/command          — submit a command
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, cast

import httpx

__all__ = ["SteelHorizonsClient", "read_discovery_file"]


def read_discovery_file(user_data_dir: Path) -> dict[str, Any]:
    """Read the connection discovery file written by the engine.

    Parameters
    ----------
    user_data_dir : Path
        Path to the user data directory (e.g. ``~/Library/Application Support/steel-horizons``
        on macOS).

    Returns
    -------
    dict
        Keys: ``protocol``, ``host``, ``port``, ``token``, ``pid``.
    """
    path = user_data_dir / "connection.json"
    with open(path, "r") as f:
        return cast(dict[str, Any], json.load(f))


class SteelHorizonsClient:
    """HTTP client for the Steel Horizons engine REST API."""

    def __init__(self, host: str = "127.0.0.1", port: int = 4880, token: str = ""):
        self._base = f"http://{host}:{port}"
        self._headers = {"Content-Type": "application/json"}
        if token:
            self._headers["Authorization"] = f"Bearer {token}"

    # ── Factory: construct from discovery file ────────────────────────

    @classmethod
    def from_discovery(cls, user_data_dir: Path | None = None) -> SteelHorizonsClient:
        """Read the discovery file and return a client configured for that session."""
        if user_data_dir is None:
            import platformdirs
            user_data_dir = Path(platformdirs.user_data_dir("steel-horizons", "steel-horizons"))
        info = read_discovery_file(user_data_dir)
        return cls(
            host=info["host"],
            port=info["port"],
            token=info["token"],
        )

    # ── HTTP helpers ──────────────────────────────────────────────────

    def _get(self, path: str) -> dict[str, Any]:
        r = httpx.get(f"{self._base}{path}", headers=self._headers)
        r.raise_for_status()
        return cast(dict[str, Any], r.json())

    def _post(self, path: str, body: dict[str, Any]) -> dict[str, Any]:
        r = httpx.post(
            f"{self._base}{path}",
            headers=self._headers,
            content=json.dumps(body),
        )
        r.raise_for_status()
        return cast(dict[str, Any], r.json())

    # ── API methods ───────────────────────────────────────────────────

    def status(self) -> dict[str, Any]:
        """GET /api/v1/status — server status."""
        return self._get("/api/v1/status")

    def state(self) -> dict[str, Any]:
        """GET /api/v1/state — full game state snapshot."""
        return self._get("/api/v1/state")

    def content(self) -> dict[str, Any]:
        """GET /api/v1/content — content catalog."""
        return self._get("/api/v1/content")

    def collection(self, name: str) -> dict[str, Any]:
        """GET /api/v1/state/{name} — collection query.

        Valid names: ``ships``, ``stations``, ``celestial_bodies``, ``research``, ``build-orders``.
        """
        return self._get(f"/api/v1/state/{name}")

    def entity(self, collection: str, entity_id: str) -> dict[str, Any]:
        """GET /api/v1/state/{collection}/{id} — single entity lookup."""
        return self._get(f"/api/v1/state/{collection}/{entity_id}")

    def command(self, envelope: dict[str, Any]) -> dict[str, Any]:
        """POST /api/v1/command — submit a command envelope.

        Parameters
        ----------
        envelope : dict
            Must contain ``id``, ``command``, and optionally ``expected_tick``.

        Returns
        -------
        dict
            Command acknowledgement.
        """
        return self._post("/api/v1/command", envelope)
