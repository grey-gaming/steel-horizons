# Steel Horizons Agent Instructions

These instructions apply to the entire repository. The canonical ordered backlog is
[`docs/IMPLEMENTATION_PLAN.md`](docs/IMPLEMENTATION_PLAN.md). Read that plan before making implementation changes.

## Scope and authority

- Phase 1 is the Rust deterministic simulation engine, authenticated loopback HTTP/WebSocket API, canonical JSON content, Python text UI/client, and agent play-tests.
- Phase 2 is the PixiJS v8/Tauri graphical client. Do not start Phase 2 until the Phase 1 completion gate and the Phase 2 technical-design gate in the implementation plan are complete.
- `docs/gdd/v2-gate-logistics.md` is future concept material and is outside the implementation scope.
- When documents overlap, use this authority order:
  1. Accepted ADRs.
  2. GDD 12 for simulation semantics and formulas.
  3. GDD 13 for serialized state shapes.
  4. GDD 14 for exact authored content and balance.
  5. Other approved GDDs for player-facing behavior and presentation.
  6. TDDs for implementation structure.
- A change to an authoritative rule must update every dependent summary, executable validation, and test in the same increment.

## Non-negotiable invariants

- Integer-only deterministic simulation state.
- Checked integer arithmetic; overflow returns typed errors and never wraps or panics.
- Stable iteration, a project-owned serialized PRNG, and recorded command order.
- In the running application, the simulation actor is the sole mutable state owner; GDD 12's phase order is fixed and execution speed never affects state.
- Exact real-time/batch, save/load, and command-replay equivalence.
- Distributed storage; no global material pool.
- No accidental material loss. Reversible actions preserve complete component/resource multisets and salvage never decays.
- Research progress and consumed-resource credit are persistent.
- Autonomous one-shot ship jobs; recurring manual routes are outside V1.
- The canonical starting state and permitted recovery actions retain a path to Gate victory.
- Generated IDs come only from serialized per-kind counters; never use UUIDs, wall-clock values, or collection lengths for simulation identity.

The following block is intentionally duplicated verbatim in `docs/IMPLEMENTATION_PLAN.md`. Changes to either copy must update both in the same change; P1-01 adds an automated synchronization check.

<!-- BEGIN PER-TURN PROTOCOL -->
## Per-turn autonomous execution protocol

On every implementation turn, follow this sequence without skipping steps:

1. **Re-establish repository context.**
   - Read root `AGENTS.md` and `docs/IMPLEMENTATION_PLAN.md`.
   - Inspect `git status --short`, recent commits, and the files/tests belonging to the current increment.
   - Preserve all unrelated or user-authored changes. Never reset or overwrite them.
2. **Select exactly one bounded increment.**
   - Resume the earliest partially completed increment; otherwise choose the earliest unchecked, unblocked increment in the active phase.
   - A direct user instruction may select a different in-scope increment, but unmet dependencies must be reported and may not be bypassed.
   - Split an increment further when useful; never combine multiple plan increments merely to make a larger change.
3. **Re-read the governing contract.**
   - Read the authoritative document sections cited by the increment and the relevant data/API/test contracts.
   - Check the specification-closure list in the plan. If deterministic behavior remains ambiguous, resolve it in a doc-only ADR/GDD/TDD change before production code. Do not silently invent gameplay, serialization, or wire semantics.
4. **Define the proof before the implementation.**
   - State the behavior and acceptance evidence for the slice.
   - Add or identify one focused test that fails for the intended reason, and run it to confirm the red state.
   - Early scaffolding that cannot have a behavioral test must have a deterministic smoke check with an explicit expected result.
   - For an explicitly doc-only Gate 0 increment, use a focused document consistency, link, or contract check instead of a behavioral test and record the exact future executable proof. Confirm the pre-change inconsistency when practical.
5. **Implement the smallest coherent change.**
   - Make only the production/content/tooling changes needed to satisfy the focused proof.
   - Keep domain/content independent of Axum and presentation code. Runtime/API code never receives mutable `GameState`; pure constructors and deterministic test harnesses may own isolated state before P1-11, after which production mutation is actor-owned.
   - Do not add speculative abstractions, unrelated cleanup, hidden fallbacks, or unrequested V2 behavior.
6. **Apply cross-cutting proof obligations.**
   - Any serialized-state change requires Serde round-trip, invariant, canonical-hash, save/load, and replay coverage.
   - Any material/inventory change requires conservation, capacity/bounds, and recovery-path coverage.
   - Any command change requires allowed/invalid lifecycle, `expected_tick`, idempotency, ordering, and malformed-input coverage; after both transports exist, run it through the shared REST/WS corpus.
   - Any content change requires validator coverage, normalized content-hash review, and affected reachability/scenario tests.
   - Any presentation change requires keyboard, non-color, UI-scale, reduced-motion, and visual-regression checks where applicable.
7. **Run verification in increasing scope.**
   - Run the focused test first, then the affected crate/package suite.
   - Run every cumulative gate activated by the plan through the current increment, in the documented order.
   - Tests must use isolated temporary data directories and explicit tick advancement; never use a developer save or `sleep()` as a simulation clock.
   - Do not accept flaky retries, skipped required tests, warnings, silent golden updates, or unexplained state-hash changes.
   - If a required tool, platform, runner, or credential is unavailable, record exactly what did and did not run; never claim the unavailable gate passed.
8. **Review the resulting diff.**
   - Check correctness, deterministic ordering, checked arithmetic, typed errors, and document traceability.
   - Confirm generated files are reproducible and no credentials, discovery tokens, saves, build outputs, or unrelated files are included.
9. **Record progress and evidence.**
   - Update the increment checkbox and evidence log in `docs/IMPLEMENTATION_PLAN.md` only when its acceptance evidence is fully satisfied.
   - Record commands run, tests/scenarios activated, and any intentional golden/hash changes.
   - Keep the plan, authoritative documents, schemas/content, and executable tests synchronized.
10. **End in a resumable state.**
    - A completed increment ends green and narrowly scoped. When the active request authorizes autonomous implementation commits, create one intentional local commit for the increment.
    - Do not push, open a PR, publish, sign, notarize, or release unless explicitly requested.
    - If incomplete, leave the checkbox open and report the exact remaining work or blocker; never claim completion based only on partial tests.
    - Report the exact verification commands and outcomes, files changed, completed increment, and next increment or blocker.

<!-- END PER-TURN PROTOCOL -->

## Verification order

Once the corresponding tooling exists, the ordinary cumulative gate is:

1. Rust formatting and Clippy with warnings denied.
2. Content/schema validation.
3. Unit and property tests.
4. Every activated deterministic scenario.
5. HTTP/WebSocket API conformance.
6. Save/load and command-log replay equivalence.
7. Python formatting, typing, and unit/integration tests.

Nightly/pre-release gates add the Python agent play-tests, supported-platform state-hash comparison, and the V1-ceiling benchmark. Use the exact repository commands established by increment P1-01 rather than substituting ad hoc commands.

## Safety and delivery boundaries

- Use isolated temporary directories for saves, discovery files, and API integration tests.
- Never expose a session bearer token in logs or command-line arguments for packaged builds.
- Never weaken loopback binding, Origin enforcement, authentication, request limits, or backpressure tests to make a test pass.
- Golden files may change only with an explained authoritative behavior/content change or a reviewed canonicalization change.
- The pending public license, asset provenance approval, signing/notarization credentials, and store credentials are release blockers, not reasons to weaken private-development checks.
