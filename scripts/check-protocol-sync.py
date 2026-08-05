#!/usr/bin/env python3
"""
Marker-based check for the duplicated per-turn protocol block.

Both AGENTS.md and docs/IMPLEMENTATION_PLAN.md contain an identical block
between <!-- BEGIN PER-TURN PROTOCOL --> and <!-- END PER-TURN PROTOCOL -->.
This script extracts both and compares them byte-for-byte.
"""

from __future__ import annotations

import os
import sys

REPO_ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))

MARKER_BEGIN = "<!-- BEGIN PER-TURN PROTOCOL -->"
MARKER_END = "<!-- END PER-TURN PROTOCOL -->"


def extract_block(path: str) -> bytes:
    with open(path, "rb") as f:
        content = f.read()

    start = content.find(MARKER_BEGIN.encode())
    end = content.find(MARKER_END.encode())

    if start < 0:
        raise ValueError(f"BEGIN marker not found in {path}")
    if end < 0:
        raise ValueError(f"END marker not found in {path}")

    # Extract from after BEGIN marker to before END marker
    start += len(MARKER_BEGIN)
    return content[start:end]


def main() -> int:
    agents_path = os.path.join(REPO_ROOT, "AGENTS.md")
    plan_path = os.path.join(REPO_ROOT, "docs", "IMPLEMENTATION_PLAN.md")

    block1 = extract_block(agents_path)
    block2 = extract_block(plan_path)

    if block1 == block2:
        print("PASS: per-turn protocol block is byte-identical between AGENTS.md and IMPLEMENTATION_PLAN.md")
        return 0
    else:
        print("FAIL: per-turn protocol block differs between AGENTS.md and IMPLEMENTATION_PLAN.md")
        if len(block1) != len(block2):
            print(f"  Size mismatch: AGENTS.md={len(block1)} bytes, plan={len(block2)} bytes")
        # Find first differing byte
        for i in range(min(len(block1), len(block2))):
            if block1[i] != block2[i]:
                context_start = max(0, i - 40)
                context_end = min(len(block1), i + 40)
                print(f"  First difference at byte {i}:")
                print(f"  AGENTS.md: {block1[context_start:context_end]!r}")
                print(f"  Plan:      {block2[context_start:context_end]!r}")
                break
        return 1


if __name__ == "__main__":
    sys.exit(main())
