# Technical Design Documents (TDDs)

This directory holds the approved Phase 1 implementation design. TDDs describe component boundaries, data flow, protocol, testing, and delivery while remaining subordinate to accepted ADRs and canonical GDDs 12–14.

Naming convention: `NN-title-with-hyphens.md`

## TDDs

| # | Title | Description |
|---|-------|-------------|
| 00 | System Architecture | Overall architecture diagram, process model, component responsibilities, data flow |
| 01 | Simulation Engine Design | Rust project structure, core types, tick phase design, save/load, determinism |
| 02 | API Protocol Design | REST endpoints, WebSocket protocol, command/event types, error handling |
| 03 | Text UI / Agent Interface | CLI/TUI client design, rendering modes, agent interaction model |
| 04 | Testing Strategy | Four-tier test architecture, unit/scenario/API/play-test design, TDD workflow |
| 05 | Build & Deployment | Cargo workspace, dependencies, cross-platform targets, CI/CD pipeline |
