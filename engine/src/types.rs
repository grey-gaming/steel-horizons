//! Primitive protocol/domain vocabulary types.
//!
//! # Documentation note
//!
//! Enum and struct field names are self-documenting per GDD 13.  We allow
//! `missing_docs` here to avoid 200+ repetitive doc comments on every
//! variant — the authoritative GDD section names in the module-level doc
//! above each type serve as the canonical reference.

#![allow(missing_docs)]
//!
//! This module defines the shared Rust types for simulation IDs, resource
//! types, lanes, lifecycle states, entity roles/states, and the various
//! tagged enums that appear in serialized state and command wire formats.
//! Every enum has explicit `#[serde(tag = "type")]` (internal tagging) so
//! its JSON representation is self-describing and deterministic.
//!
//! ## Ord implementations
//!
//! All C-like enums derive `Ord` via the standard `#[derive(...)]` using the
//! order given in the source (which matches GDD 13's canonical presentation).
//! `ResourceType` and `LaneId` have manual `Ord` that follows the canonical
//! resource/lane order from GDD 14 §Canonical Refining Recipes and GDD 13's
//! lane definitions.  This ensures `BTreeMap`/`BTreeSet` iteration is stable.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ─── Resource type ────────────────────────────────────────────────────

/// The 25 resource types in canonical order (raw, refined, component).
///
/// Serialized as a camelCase string (e.g. `"metalOre"`).  This enum's
/// `Ord` follows the GDD 14 resource catalog order so that sparse maps
/// iterate deterministically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResourceType {
    // ── Raw resources ──
    MetalOre,
    CarbonSoil,
    SiliconDust,
    VolcanicSulfur,
    WaterIce,
    FrozenGases,
    Helium3,
    RareEarthMinerals,
    CrystalDeposits,
    // ── Refined resources ──
    Metals,
    CarbonFiber,
    SiliconWafers,
    Chemicals,
    Fuel,
    Alloys,
    Optics,
    ReactorRods,
    // ── Components ──
    StructuralFrame,
    PowerCore,
    ControlSystem,
    DriveAssembly,
    CargoModule,
    ResearchLab,
    ConstructionBay,
    GateNode,
}

impl ResourceType {
    /// The total number of resource variants.
    pub const COUNT: usize = 25;

    /// Return the canonical ordering rank (0‑based).
    fn rank(self) -> u8 {
        match self {
            Self::MetalOre => 0,
            Self::CarbonSoil => 1,
            Self::SiliconDust => 2,
            Self::VolcanicSulfur => 3,
            Self::WaterIce => 4,
            Self::FrozenGases => 5,
            Self::Helium3 => 6,
            Self::RareEarthMinerals => 7,
            Self::CrystalDeposits => 8,
            Self::Metals => 9,
            Self::CarbonFiber => 10,
            Self::SiliconWafers => 11,
            Self::Chemicals => 12,
            Self::Fuel => 13,
            Self::Alloys => 14,
            Self::Optics => 15,
            Self::ReactorRods => 16,
            Self::StructuralFrame => 17,
            Self::PowerCore => 18,
            Self::ControlSystem => 19,
            Self::DriveAssembly => 20,
            Self::CargoModule => 21,
            Self::ResearchLab => 22,
            Self::ConstructionBay => 23,
            Self::GateNode => 24,
        }
    }
}

impl PartialOrd for ResourceType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

// ─── Lane ID ──────────────────────────────────────────────────────────

/// Orbital lane identifiers in increasing orbital radius order.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum LaneId {
    Inner,
    Habitable,
    Outer,
    Fringe,
}

// ─── Game lifecycle ───────────────────────────────────────────────────

/// Lifecycle states of the game session.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum GameLifecycle {
    Unloaded,
    Loading,
    Paused,
    Running,
    Advancing,
    Won,
}

// ─── Entity roles and states ──────────────────────────────────────────

/// Role of a ship.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ShipRole {
    Cargo,
    Construction,
    Research,
}

/// Possible states of a ship.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ShipState {
    Idle,
    Holding,
    InTransit,
    Building,
    Surveying,
    PoweringResearch,
    AwaitingRescue,
}

/// Station type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum StationType {
    Hub,
    Mining,
    Refinery,
    Construction,
    Research,
}

/// Celestial body type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum BodyType {
    Planet,
    Moon,
    AsteroidBelt,
}

/// Sub-type for planets.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum PlanetSubtype {
    RockyTerran,
    Volcanic,
    IceWorld,
    GasGiant,
}

/// Direction along a lane arc.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ArcDirection {
    Clockwise,
    CounterClockwise,
}

// ─── Destination references ───────────────────────────────────────────

/// A destination for ship travel or logistics.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum DestinationRef {
    Body { body_id: crate::id::BodyId },
    Station { station_id: crate::id::StationId },
    Salvage { salvage_id: crate::id::SalvageId },
    GateSite,
}

/// A destination for cargo delivery (inventory transfers).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum InventoryDestinationRef {
    Station { station_id: crate::id::StationId },
    BuildOrder { order_id: crate::id::BuildOrderId },
    Evacuation { order_id: crate::id::BuildOrderId },
    GateSite,
}

/// A source for cargo pickup.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum InventorySourceRef {
    Station { station_id: crate::id::StationId },
    Salvage { salvage_id: crate::id::SalvageId },
}

// ─── Ship job ─────────────────────────────────────────────────────────

/// Stage of a Transport job.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum TransportStage {
    ToPickup,
    ToDelivery,
}

/// The job a ship is performing.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ShipJob {
    Idle,
    Transport {
        reservation_id: crate::id::ReservationId,
        source: InventorySourceRef,
        destination: InventoryDestinationRef,
        resource: ResourceType,
        amount: u32,
        stage: TransportStage,
    },
    Refuel {
        station_id: crate::id::StationId,
    },
    Build {
        order_id: crate::id::BuildOrderId,
    },
    Upgrade {
        order_id: crate::id::BuildOrderId,
    },
    Demolish {
        order_id: crate::id::BuildOrderId,
    },
    Survey {
        order_id: crate::id::SurveyOrderId,
    },
    DockForResearch {
        station_id: crate::id::StationId,
        tech_id: crate::id::TechId,
    },
    Rescue {
        hub_id: crate::id::StationId,
        dispatch_ticks_remaining: u16,
    },
}

// ─── Build targets and states ─────────────────────────────────────────

/// What a build order is constructing.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum BuildTarget {
    Ship {
        hub_id: crate::id::StationId,
        role: ShipRole,
        tier: u8,
    },
    Station {
        body_id: crate::id::BodyId,
        orbit_ring: u8,
        slot: u8,
        station_type: StationType,
        tier: u8,
    },
    Upgrade {
        station_id: crate::id::StationId,
        target_tier: u8,
    },
    Demolish {
        station_id: crate::id::StationId,
        recovery_hub_id: crate::id::StationId,
    },
}

/// Build order lifecycle states.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum BuildState {
    AwaitingMaterials,
    Ready,
    Traveling,
    Evacuating,
    Building,
    Complete,
    Cancelling,
    Cancelled,
}

// ─── Research ─────────────────────────────────────────────────────────

/// Research project lifecycle states.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ResearchState {
    AwaitingMaterials,
    Ready,
    Active,
    Paused,
    Complete,
}

/// Reason a research project was paused.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ResearchPauseReason {
    Manual,
    NoResearchShip,
    FacilityUnavailable,
}

// ─── Survey orders ────────────────────────────────────────────────────

/// Survey order lifecycle states.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum SurveyOrderState {
    Queued,
    Assigned,
    Complete,
    Cancelled,
}

// ─── Logistics reservations ───────────────────────────────────────────

/// Reservation lifecycle states.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum ReservationState {
    AwaitingPickup,
    Loaded,
    Delivered,
    Released,
}

// ─── Gate ─────────────────────────────────────────────────────────────

/// Space Gate assembly phases.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum GatePhase {
    SitePreparation,
    FrameAssembly,
    PowerIntegration,
    Activation,
}

// ─── Commands ─────────────────────────────────────────────────────────

/// Outcome of a command.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CommandOutcome {
    Accepted,
    Applied,
    Rejected,
}

/// When a command takes effect.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CommandApplicationBoundary {
    PausedImmediate,
    ScheduledTick,
}

/// Typed error detail values.
///
/// Per GDD 13, `ErrorDetail` can be a string, bool, integer, null, or
/// arrays of strings or integers.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(untagged)]
pub enum ErrorDetail {
    String(String),
    Bool(bool),
    U64(u64),
    Null,
    StringArray(Vec<String>),
    U64Array(Vec<u64>),
}

/// Structured rejection information for a failed command.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CommandRejection {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: std::collections::BTreeMap<String, ErrorDetail>,
}

/// Result payload produced by an accepted or applied command.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum CommandResult {
    None,
    BuildOrderCreated { order_id: crate::id::BuildOrderId },
    SurveyOrderCreated { order_id: crate::id::SurveyOrderId },
    ResearchProjectCreated { tech_id: crate::id::TechId },
    ResearchProjectUpdated { tech_id: crate::id::TechId },
    GateAssemblyStarted,
    AdvanceTicksCompleted { resulting_tick: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::*;
    use std::collections::BTreeMap;

    // ─── Resource type ────────────────────────────────────────────────

    /// ResourceType round-trips through JSON with camelCase naming.
    #[test]
    fn resource_type_round_trip() {
        let cases = [
            ResourceType::MetalOre,
            ResourceType::Fuel,
            ResourceType::GateNode,
        ];
        for &ty in &cases {
            let json = serde_json::to_string(&ty).unwrap();
            let back: ResourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(ty, back);
        }
        // Check specific camelCase string
        let json = serde_json::to_string(&ResourceType::MetalOre).unwrap();
        assert_eq!(json, "\"metalOre\"");
        let json = serde_json::to_string(&ResourceType::RareEarthMinerals).unwrap();
        assert_eq!(json, "\"rareEarthMinerals\"");
    }

    /// ResourceType has the canonical GDD 14 ordering.
    #[test]
    fn resource_type_ordering() {
        let mut resources = [
            ResourceType::GateNode,
            ResourceType::MetalOre,
            ResourceType::Fuel,
            ResourceType::CarbonSoil,
        ];
        resources.sort();
        let ordered: Vec<ResourceType> = resources.into();
        assert_eq!(
            ordered,
            [
                ResourceType::MetalOre,
                ResourceType::CarbonSoil,
                ResourceType::Fuel,
                ResourceType::GateNode,
            ]
        );
    }

    /// ResourceType::COUNT matches the expected 25 variants.
    #[test]
    fn resource_type_count() {
        assert_eq!(ResourceType::COUNT, 25);
    }

    // ─── Lane ID ──────────────────────────────────────────────────────

    /// LaneId round-trips with camelCase.
    #[test]
    fn lane_id_round_trip() {
        let json = serde_json::to_string(&LaneId::Habitable).unwrap();
        assert_eq!(json, "\"habitable\"");
        let back: LaneId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, LaneId::Habitable);
    }

    /// LaneId ordering follows orbital radius.
    #[test]
    fn lane_id_ordering() {
        assert!(LaneId::Inner < LaneId::Habitable);
        assert!(LaneId::Habitable < LaneId::Outer);
        assert!(LaneId::Outer < LaneId::Fringe);
    }

    // ─── Game lifecycle ───────────────────────────────────────────────

    #[test]
    fn game_lifecycle_round_trip() {
        let json = serde_json::to_string(&GameLifecycle::Paused).unwrap();
        assert_eq!(json, "\"paused\"");
        let back: GameLifecycle = serde_json::from_str(&json).unwrap();
        assert_eq!(back, GameLifecycle::Paused);

        let json = serde_json::to_string(&GameLifecycle::Advancing).unwrap();
        assert_eq!(json, "\"advancing\"");
    }

    // ─── Entity enums ─────────────────────────────────────────────────

    #[test]
    fn entity_enum_round_trips() {
        // ShipRole
        let json = serde_json::to_string(&ShipRole::Cargo).unwrap();
        assert_eq!(json, "\"cargo\"");
        let back: ShipRole = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ShipRole::Cargo);

        // ShipState
        let json = serde_json::to_string(&ShipState::InTransit).unwrap();
        assert_eq!(json, "\"inTransit\"");
        let back: ShipState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ShipState::InTransit);

        // StationType
        let json = serde_json::to_string(&StationType::Construction).unwrap();
        assert_eq!(json, "\"construction\"");
        let back: StationType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, StationType::Construction);

        // BodyType
        let json = serde_json::to_string(&BodyType::AsteroidBelt).unwrap();
        assert_eq!(json, "\"asteroidBelt\"");
        let back: BodyType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, BodyType::AsteroidBelt);

        // PlanetSubtype
        let json = serde_json::to_string(&PlanetSubtype::RockyTerran).unwrap();
        assert_eq!(json, "\"rockyTerran\"");
        let back: PlanetSubtype = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PlanetSubtype::RockyTerran);

        // ArcDirection
        let json = serde_json::to_string(&ArcDirection::Clockwise).unwrap();
        assert_eq!(json, "\"clockwise\"");
    }

    // ─── Tagged reference enums ───────────────────────────────────────

    #[test]
    fn destination_ref_tagged() {
        let d = DestinationRef::Body {
            body_id: BodyId("planet_haven".into()),
        };
        let json = serde_json::to_string(&d).unwrap();
        assert_eq!(json, r#"{"type":"body","body_id":"planet_haven"}"#);
        let back: DestinationRef = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);

        // GateSite (no fields)
        let gs = DestinationRef::GateSite;
        let json = serde_json::to_string(&gs).unwrap();
        assert_eq!(json, r#"{"type":"gateSite"}"#);
        let back: DestinationRef = serde_json::from_str(&json).unwrap();
        assert_eq!(gs, back);
    }

    #[test]
    fn inventory_destination_ref_tagged() {
        let d = InventoryDestinationRef::Station {
            station_id: StationId("hub_haven".into()),
        };
        let json = serde_json::to_string(&d).unwrap();
        assert_eq!(json, r#"{"type":"station","station_id":"hub_haven"}"#);
        let back: InventoryDestinationRef = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn inventory_source_ref_tagged() {
        let s = InventorySourceRef::Station {
            station_id: StationId("hub_haven".into()),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#"{"type":"station","station_id":"hub_haven"}"#);
        let back: InventorySourceRef = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    // ─── Ship job ─────────────────────────────────────────────────────

    #[test]
    fn ship_job_tagged() {
        let idle = ShipJob::Idle;
        let json = serde_json::to_string(&idle).unwrap();
        assert_eq!(json, r#"{"type":"idle"}"#);
        let back: ShipJob = serde_json::from_str(&json).unwrap();
        assert_eq!(idle, back);

        let transport = ShipJob::Transport {
            reservation_id: ReservationId("r1".into()),
            source: InventorySourceRef::Station {
                station_id: StationId("src".into()),
            },
            destination: InventoryDestinationRef::Station {
                station_id: StationId("dst".into()),
            },
            resource: ResourceType::Metals,
            amount: 50,
            stage: TransportStage::ToPickup,
        };
        let json = serde_json::to_string(&transport).unwrap();
        let back: ShipJob = serde_json::from_str(&json).unwrap();
        assert_eq!(transport, back);
    }

    // ─── State enum round trips ──────────────────────────────────────

    /// BuildState round-trips every variant.
    #[test]
    fn build_state_all_variants() {
        let states = [
            BuildState::AwaitingMaterials,
            BuildState::Ready,
            BuildState::Traveling,
            BuildState::Evacuating,
            BuildState::Building,
            BuildState::Complete,
            BuildState::Cancelling,
            BuildState::Cancelled,
        ];
        for &s in &states {
            let json = serde_json::to_string(&s).unwrap();
            let back: BuildState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    /// ResearchState round-trips every variant.
    #[test]
    fn research_state_all_variants() {
        let states = [
            ResearchState::AwaitingMaterials,
            ResearchState::Ready,
            ResearchState::Active,
            ResearchState::Paused,
            ResearchState::Complete,
        ];
        for &s in &states {
            let json = serde_json::to_string(&s).unwrap();
            let back: ResearchState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    /// SurveyOrderState round-trips every variant.
    #[test]
    fn survey_order_state_all_variants() {
        let states = [
            SurveyOrderState::Queued,
            SurveyOrderState::Assigned,
            SurveyOrderState::Complete,
            SurveyOrderState::Cancelled,
        ];
        for &s in &states {
            let json = serde_json::to_string(&s).unwrap();
            let back: SurveyOrderState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    /// ReservationState round-trips every variant.
    #[test]
    fn reservation_state_all_variants() {
        let states = [
            ReservationState::AwaitingPickup,
            ReservationState::Loaded,
            ReservationState::Delivered,
            ReservationState::Released,
        ];
        for &s in &states {
            let json = serde_json::to_string(&s).unwrap();
            let back: ReservationState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    /// GatePhase round-trips every variant.
    #[test]
    fn gate_phase_all_variants() {
        let phases = [
            GatePhase::SitePreparation,
            GatePhase::FrameAssembly,
            GatePhase::PowerIntegration,
            GatePhase::Activation,
        ];
        for &p in &phases {
            let json = serde_json::to_string(&p).unwrap();
            let back: GatePhase = serde_json::from_str(&json).unwrap();
            assert_eq!(p, back);
        }
    }

    /// ResearchPauseReason round-trips every variant.
    #[test]
    fn research_pause_reason_all_variants() {
        let reasons = [
            ResearchPauseReason::Manual,
            ResearchPauseReason::NoResearchShip,
            ResearchPauseReason::FacilityUnavailable,
        ];
        for &r in &reasons {
            let json = serde_json::to_string(&r).unwrap();
            let back: ResearchPauseReason = serde_json::from_str(&json).unwrap();
            assert_eq!(r, back);
        }
    }

    // ─── Build target ─────────────────────────────────────────────────

    #[test]
    fn build_target_tagged() {
        let ship = BuildTarget::Ship {
            hub_id: StationId("hub_haven".into()),
            role: ShipRole::Cargo,
            tier: 1,
        };
        let json = serde_json::to_string(&ship).unwrap();
        let back: BuildTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(ship, back);
    }

    // ─── Transport stage ──────────────────────────────────────────────

    #[test]
    fn transport_stage_round_trip() {
        let cases = [TransportStage::ToPickup, TransportStage::ToDelivery];
        for &stage in &cases {
            let json = serde_json::to_string(&stage).unwrap();
            let back: TransportStage = serde_json::from_str(&json).unwrap();
            assert_eq!(stage, back);
        }
        let json = serde_json::to_string(&TransportStage::ToPickup).unwrap();
        assert_eq!(json, "\"toPickup\"");
        let json = serde_json::to_string(&TransportStage::ToDelivery).unwrap();
        assert_eq!(json, "\"toDelivery\"");
    }

    // ─── Command types ────────────────────────────────────────────────

    #[test]
    fn command_outcome_round_trip() {
        let json = serde_json::to_string(&CommandOutcome::Accepted).unwrap();
        assert_eq!(json, "\"accepted\"");
        let back: CommandOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CommandOutcome::Accepted);
    }

    #[test]
    fn command_application_boundary_round_trip() {
        let json = serde_json::to_string(&CommandApplicationBoundary::ScheduledTick).unwrap();
        assert_eq!(json, "\"scheduledTick\"");
        let back: CommandApplicationBoundary = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CommandApplicationBoundary::ScheduledTick);
    }

    #[test]
    fn command_result_tagged() {
        let cr = CommandResult::AdvanceTicksCompleted {
            resulting_tick: 100,
        };
        let json = serde_json::to_string(&cr).unwrap();
        assert_eq!(
            json,
            r#"{"type":"advanceTicksCompleted","resulting_tick":100}"#
        );
        let back: CommandResult = serde_json::from_str(&json).unwrap();
        assert_eq!(cr, back);
    }

    #[test]
    fn error_detail_variants() {
        let cases: Vec<ErrorDetail> = vec![
            ErrorDetail::String("test".into()),
            ErrorDetail::Bool(true),
            ErrorDetail::U64(42),
            ErrorDetail::Null,
            ErrorDetail::StringArray(vec!["a".into(), "b".into()]),
            ErrorDetail::U64Array(vec![1, 2, 3]),
        ];
        for ed in &cases {
            let json = serde_json::to_string(ed).unwrap();
            let back: ErrorDetail = serde_json::from_str(&json).unwrap();
            assert_eq!(*ed, back);
        }
    }

    #[test]
    fn command_rejection_round_trip() {
        let r = CommandRejection {
            code: "BAD_REQUEST".into(),
            message: "invalid command".into(),
            details: BTreeMap::from([("field".into(), ErrorDetail::String("ticks".into()))]),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: CommandRejection = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    // ─── Unknown / invalid field rejection ────────────────────────────

    #[test]
    fn unknown_field_rejected() {
        // DestinationRef should reject unknown fields
        let bad = r#"{"type":"body","body_id":"x","unknown":true}"#;
        let result: Result<DestinationRef, _> = serde_json::from_str(bad);
        assert!(result.is_err(), "unknown field should be rejected");
    }

    #[test]
    fn missing_variant_field_rejected() {
        // DestinationRef::Body requires body_id
        let bad = r#"{"type":"body"}"#;
        let result: Result<DestinationRef, _> = serde_json::from_str(bad);
        assert!(result.is_err(), "missing variant field should be rejected");
    }
}
