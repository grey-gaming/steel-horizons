---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-03
audience: Development team, stakeholders
---

# Steel Horizons — Game Design Document (GDD)

## Overview

**Steel Horizons** is a space transportation and logistics game inspired by OpenTTD, set in a single star system. You don't control ships directly — you build the infrastructure that makes trade and industry possible. Your goal: collect resources, refine them, build factories, construct ships and stations, and research new technology until you can build a **Space Gate** that opens travel to other star systems.

## Core Loop

1. **Mine / Harvest** raw resources from planets, moons, and asteroid belts
2. **Transport** raw resources to your stations and factories
3. **Refine / Build** components and structures at factories
4. **Research** new technology by spending resources
5. **Expand** — build more ships, stations, factories to handle bigger logistics
6. Repeat until you can construct the Space Gate

There is no money. Everything is resource-driven — the more materials you collect and the smarter you build your logistics network, the more you can construct.

## Player Role

You are a **space logistics director**. You don't fly a ship. You:
- Place stations and factories
- Choose what to refine and build
- Direct research priorities
- Decide when to expand (build more ships, upgrade stations)

The ships and stations operate autonomously once built — your job is to design the network.

## V1 Scope

The game takes place in **one star system**. The system contains:
- A central star
- Multiple planets (each with different surface conditions and resource profiles)
- Moons orbiting some planets
- Asteroid belts between orbital lanes

The player starts with a single small construction ship, a basic station, and a **Starter Kit** of pre-assembled components (Drive Assembly, Research Lab, Construction Bay, Power Cores, Control System, Cargo Modules) that enable building the first ships and factories. The system is partially **fogged** — you can see celestial bodies but their resource contents are hidden until surveyed. Over the course of V1, the player will:
- Explore and survey the system to reveal resources
- Establish mining operations on multiple bodies
- Build a logistics network of cargo ships and stations
- Construct factories to refine materials
- Progress through a research tree
- Build a **Space Gate** as the final goal

The Space Gate is the top of the tech tree. Completing it enables V2 — travel to other systems. But within V1, the Gate is the victory condition: once it's operational, you've "won" this system.

## Difficulty Philosophy — No Failure State

Steel Horizons has **no failure state**. There is no game over, no ship destruction, no pirates, no permanent resource loss. The challenge is purely **logistical** — can you design a network that moves materials efficiently?

If production backs up or routes are inefficient, things slow down or stop — but nothing is destroyed. You can always recover by redesigning routes, adding capacity, or researching better technology. The game is a **cozy logistical puzzle**, not a survival game.

The difficulty comes from:
- **Resource availability** — deposits are finite in throughput, forcing you to expand to new bodies
- **Placement strategy** — station position determines route efficiency
- **Capacity planning** — too few ships = bottlenecks, too many = wasted components
- **Tech prioritization** — which research to pursue first shapes your options

There is always a solution. The game rewards smart network design, not twitch reflexes.

## Design Influences

- **OpenTTD** — route-based logistics, cargo types, progressive vehicle tiers, station networks
- **Thea 2** — resource-based crafting, tech tree, no currency
- **Kerbal Space Program** — celestial body geography, orbital travel
- **Factorio** — factory chains, progressive automation

## Visual Tone

Clean, functional sci-fi. Not photorealistic — readable maps with clear icons for cargo types, ship classes, and station functions. Celestial bodies rendered at a scale where the player can see routes and stations clearly. The map is the main interface.
