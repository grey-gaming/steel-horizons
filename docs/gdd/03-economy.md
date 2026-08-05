---
status: Approved
owner: Product Owner
last-reviewed: 2026-08-04
---

# Economy & Resources

There is no money. The economy transforms extracted resources into refined goods, components, ships, stations, research, and finally the Space Gate. Exact quantities and costs are canonical in [14-authored-content.md](./14-authored-content.md); this document owns the economy's player-facing rules.

## Bootstrap Economy

Hub Haven begins with a pre-assembled deployment kit containing the complete loose component costs for:

1. A Tier-1 Mining Station
2. A Tier-1 Refinery
3. The first Cargo Ship
4. A Research Ship and Research Station
5. A first Construction Factory after Factory Automation

It also contains a third Drive Assembly reserved for a second Courier/Hauler after the player produces that ship's Frame and Cargo Module(s). These pre-assembled units may be used before their production recipes are researched. Once the kit is consumed, replacements follow normal technology and recipe rules. The machine-readable starting fixture and mandatory bootstrap/first-five-hour scenario tests verify this sequence.

## Resource Tiers

### Raw Resources

| Resource | Principal sources | Principal use |
|----------|-------------------|---------------|
| Metal Ore | Haven, Glint, The Veil | Metals |
| Carbon Soil | Haven, Boreas, The Veil | Carbon Fiber |
| Silicon Dust | Haven, The Veil | Silicon Wafers |
| Volcanic Sulfur | Pyre, The Veil | Chemicals |
| Water Ice | Boreas, The Veil | Chemicals |
| Frozen Gases | Boreas, The Veil | Fuel |
| Helium-3 | Rime | Reactor Rods |
| Rare Earth Minerals | Pyre, Rime | Alloys, Reactor Rods |
| Crystal Deposits | Pyre, Glint, The Veil | Optics |

Deposits are visible only at or above their authored survey depth. Planet and moon deposits are finite. Belt deposits are renewable density fields and guarantee a permanent source of common construction and fuel-chain inputs.

### Refined Goods

| Good | Inputs, summarized | Technology |
|------|--------------------|------------|
| Metals | Metal Ore | Basic Refining |
| Carbon Fiber | Carbon Soil | Basic Refining |
| Silicon Wafers | Silicon Dust | Basic Refining |
| Chemicals | Volcanic Sulfur + Water Ice | Advanced Refining |
| Fuel | Frozen Gases + Chemicals | Advanced Refining |
| Alloys | Metals + Rare Earth Minerals | Alloy Smelting |
| Optics | Crystal Deposits + Silicon Wafers | Sensor Systems |
| Reactor Rods | Helium-3 + Rare Earth Minerals | Fusion Power |

### Components

| Component | Role |
|-----------|------|
| Structural Frame | Hull and station structure |
| Power Core | Permanent station power |
| Control System | Automation and control |
| Drive Assembly | Ship propulsion |
| Cargo Module | Ship and station storage |
| Research Lab | Research ships and stations |
| Construction Bay | Construction ships and factories |
| Gate Node | Space Gate segment |

Hub Haven can assemble any unlocked non-Gate component at one unit per 30 ticks. A Construction Factory assembles unlocked components in 10-tick production slots. Gate Nodes additionally require a Tier-4 Construction Factory.

## Production Rules

- Each recipe reserves all integer inputs at cycle start.
- Changing/clearing a Processing slot with `SetProductionRecipe` cancels that cycle and releases its reserved inputs unchanged. An OutputBlocked slot rejects a recipe change until its completed batch transfers, because that batch remains an output asset.
- A completed cycle consumes reserved inputs and transfers complete integer outputs atomically.
- If the output buffer is full, the completed batch waits without consuming another batch of inputs.
- Higher factory tiers add parallel production slots; they do not change recipe stoichiometry.
- A station can configure one recipe per production slot.
- Recipe/mining configuration creates only non-destructive buffer changes using the automatic defaults in GDD 14. If even the one-batch minimum cannot fit, the command is rejected and names the required free capacity.
- Alloys, Optics, and Reactor Rods have exact inverse reclamation recipes at the same Tier-3 refinery. Overproduction cannot permanently trap finite critical raw inputs.
- Every component has an exact inverse disassembly recipe at the same technology/facility tier, including surplus Gate Nodes.

These rules avoid partial-input loss and make conservation testable.

## Construction and Recycling

Ordinary build orders first move matching unreserved components already in the source Hub atomically into order staging, then expose only the missing map as demand at that Hub. Normal per-ship logistics reservations claim external supply as matches are assigned. Cargo Ships deliver into the BuildOrder through the Hub's docks; staged components do not occupy a general buffer. This local-allocation rule is what lets the deployment kit build the first Mine, Refinery, and Cargo Ship before cargo transport exists. A Construction Ship loads the full multi-component payload before travelling to a station/upgrade site. Ship orders remain at the Hub shipyard, and the Gate uses its separate fixed-site manifest. Components become part of the completed entity; they are not destroyed.

- Cancelling a build releases all reserved and delivered components.
- Queueing demolition stops new work, releases cancellable reservations, and creates a permanent empty recovery cache at the selected Hub. Source-locked evacuation jobs move every ordinary buffer—including Fuel and released research inputs—from the target station to that cache through normal Cargo Ship travel. Loaded inbound deliveries finish and join the evacuation manifest. Demolition work begins only after inventory, reservations, and docks are clear; it then returns the full installed component recipe to the assigned Construction Ship. Ship scrapping is available only while docked at a Hub; cargo, Fuel, and the component recipe move into available Hub compartments, with every overflow placed in Hub-side salvage.
- If the recovery Hub lacks capacity, the returning Construction Ship deposits the components in a permanent salvage cache at that Hub. A demolition never strands components at the demolished station's former site.
- Upgrades require only the canonical target-tier cost minus the installed current-tier cost.

This full recovery is a V1 design rule. Time spent building is not refunded, but material value is never accidentally lost.

## Research Spending

Research materials are the principal intentional material sink. All required materials are reserved before a project becomes Active, then consumed deterministically over its duration.

Research is paused, not abandoned. Paused projects retain completed ticks and consumed-resource credit. With `release_unused = false`, delivered materials stay reserved in the current facility; with `true`, the reservation is removed and the same units remain in that facility's ordinary buffers. Demolition automatically performs the latter, then waits in Evacuating while its source-locked recovery jobs move the inventory to the Hub-side cache. The detached project demands its still-required amounts when assigned to a new facility. No recovery action teleports material or wastes finite Gate-critical resources.

## Fuel

Ships consume Fuel while moving. Consumption uses actual distance and `base_mass + payload_amount`, where payload is Cargo Ship cargo, the total Construction Ship build hold, or zero for Research Ships. Empty ships still consume fuel and the final arrival tick cannot overcharge. Exact integer computation is defined in GDD 12.

Fuel rules:

- Fuel capacity is defined per ship tier in GDD 14.
- Every empty, unreserved idle ship role below `max_fuel` refuels automatically from unreserved station Fuel before ordinary work. Direct refueling is local consumption and may draw below the Cargo export floor, but never consumes Fuel protected by an `AwaitingPickup` logistics reservation or a production/research hold. Phase 9 uses the phase-8 post-debit fact and subtracts each newly created Cargo Fuel reservation from its local budget.
- A ship keeps a 10% route reserve and refuses jobs it cannot complete safely.
- Refuel reachability uses the exact movement/Fuel calculation, including serialized remainders and Life Support. Final movement Fuel is charged before the dock transaction transfers available stock; a transfer may be partial or zero after Ship-ID-ordered contention.
- Ships in a dock holding pattern consume no fuel.
- Hub Haven begins with 200 Fuel.
- A below-full ship already docked at a Hub waits there if no direct-refuel stock exists. Otherwise an empty, unreserved ship with no reachable Fuel uses the nearest existing Station Hub's solar rescue tug (route distance, then Hub ID) after 300 ticks and is towed directly home at half its base speed without consuming ship Fuel. It docks with exactly its pre-tow Fuel and both remainders unchanged, then retries normal refueling. The tug cannot carry cargo or contribute to logistics/construction, so it is recovery—not free production supply. Normal job feasibility guarantees a loaded/reserved ship never enters this state.

Renewable belt deposits include the inputs for Fuel, so a functioning late-game network cannot permanently exhaust propulsion resources.

## Power

Every completed station includes its required installed Power Core. It is a permanent construction component, not recurring fuel; V1 has no separate powered/unpowered runtime state.

Grid Power represents the high-capacity distribution knowledge required by the Space Gate. It is a progression prerequisite in V1; station-to-station power sharing and power-failure simulation are post-V1. Every V1 station therefore still requires its installed Power Core and has no recurring power input.

The Space Gate consumes one Reactor Rod for its final V1 activation. Activation immediately wins the scenario; continuous post-victory Gate upkeep is outside V1.

## Distributed Storage

There is no global inventory. Materials reside in station buffers, ships, active reservations, production-slot reservations, build orders, research projects, salvage caches, or finite deposits. Every conservation test accounts for all of those locations.

Fuel exists only in ships, salvage, or a station's separate Fuel compartment; it never occupies general cargo. Refinery Fuel output, Fuel logistics, Drive Assembly input, and Life Support research all read/write that compartment. Every completed station begins with an empty authored-capacity compartment except Hub Haven, which begins full. Its default demand threshold is 20% and export threshold is 50%, preventing simultaneous supply/demand. For general cargo, the sum of configured per-resource buffer maxima may not exceed the station's total capacity.

## Solvability

The economy satisfies the “always solvable” promise through four enforceable mechanisms:

1. A validated deployment kit makes the starting sequence reachable.
2. Reversible actions recover 100% of components.
3. Research progress and consumed credit persist when paused.
4. Authored critical-resource headroom, reversible critical refining, and renewable common inputs cover all technologies and the Gate.

CI validates the technology graph, recipe reachability, bootstrap sequence, conservation invariants, and full victory path.
