//! Schema export binary.
//!
//! Generates deterministic JSON Schema files for every root DTO in the
//! simulation engine.  Run via `cargo run -p steel-horizons-engine
//! --bin schema_export -- <output-dir>`.
//!
//! ## Authoritative references
//!
//! - GDD 13 §Schema Generation Ownership

#![deny(unsafe_code)]
#![forbid(noop_method_call)]

use std::fs;
use std::path::PathBuf;

use steel_horizons_engine::command::*;
use steel_horizons_engine::content::*;
use steel_horizons_engine::lifecycle::*;
use steel_horizons_engine::state::*;

/// Generate a schema file at `dest` for concrete type `T`.
macro_rules! write_schema {
    ($dest:expr, $name:expr, $ty:ty) => {{
        let schema = schemars::schema_for!($ty);
        let json = serde_json::to_string_pretty(&schema)
            .unwrap_or_else(|e| panic!("Failed to serialize schema for {}: {}", $name, e));
        fs::write($dest, json).unwrap_or_else(|e| panic!("Failed to write {}: {}", $name, e));
        eprintln!("  wrote  {}", $dest.display());
    }};
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: schema_export <output-dir>");
        std::process::exit(1);
    }

    let out_dir = PathBuf::from(&args[1]);
    fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("Cannot create output dir {}: {}", out_dir.display(), e));

    eprintln!(
        "[schema_export] Generating schemas in {}",
        out_dir.display()
    );

    // ── State DTOs ──
    write_schema!(&out_dir.join("GameState.json"), "GameState", GameState);
    write_schema!(
        &out_dir.join("GameSnapshot.json"),
        "GameSnapshot",
        GameSnapshot
    );
    write_schema!(
        &out_dir.join("SystemPosition.json"),
        "SystemPosition",
        SystemPosition
    );
    write_schema!(
        &out_dir.join("TravelSegment.json"),
        "TravelSegment",
        TravelSegment
    );
    write_schema!(&out_dir.join("TravelPlan.json"), "TravelPlan", TravelPlan);
    write_schema!(&out_dir.join("Buffer.json"), "Buffer", Buffer);
    write_schema!(
        &out_dir.join("ProductionSlot.json"),
        "ProductionSlot",
        ProductionSlot
    );
    write_schema!(
        &out_dir.join("MiningTarget.json"),
        "MiningTarget",
        MiningTarget
    );
    write_schema!(&out_dir.join("Station.json"), "Station", Station);
    write_schema!(&out_dir.join("Ship.json"), "Ship", Ship);
    write_schema!(
        &out_dir.join("ResourceDeposit.json"),
        "ResourceDeposit",
        ResourceDeposit
    );
    write_schema!(
        &out_dir.join("CelestialBody.json"),
        "CelestialBody",
        CelestialBody
    );
    write_schema!(
        &out_dir.join("RationalRemainder.json"),
        "RationalRemainder",
        RationalRemainder
    );
    write_schema!(
        &out_dir.join("ResearchProject.json"),
        "ResearchProject",
        ResearchProject
    );
    write_schema!(
        &out_dir.join("SurveyOrder.json"),
        "SurveyOrder",
        SurveyOrder
    );
    write_schema!(&out_dir.join("BuildOrder.json"), "BuildOrder", BuildOrder);
    write_schema!(
        &out_dir.join("SalvageCache.json"),
        "SalvageCache",
        SalvageCache
    );
    write_schema!(&out_dir.join("GateBuild.json"), "GateBuild", GateBuild);
    write_schema!(
        &out_dir.join("Reservation.json"),
        "Reservation",
        Reservation
    );
    write_schema!(
        &out_dir.join("BottleneckTracker.json"),
        "BottleneckTracker",
        BottleneckTracker
    );
    write_schema!(&out_dir.join("RNGState.json"), "RNGState", RNGState);
    write_schema!(&out_dir.join("IdCounters.json"), "IdCounters", IdCounters);
    write_schema!(
        &out_dir.join("CommandRecord.json"),
        "CommandRecord",
        CommandRecord
    );

    // ── Content DTOs ──
    write_schema!(
        &out_dir.join("DefinitionsCatalog.json"),
        "DefinitionsCatalog",
        DefinitionsCatalog
    );
    write_schema!(
        &out_dir.join("StartingScenario.json"),
        "StartingScenario",
        StartingScenario
    );
    write_schema!(
        &out_dir.join("ContentCatalog.json"),
        "ContentCatalog",
        ContentCatalog
    );
    write_schema!(
        &out_dir.join("RecipeDefinition.json"),
        "RecipeDefinition",
        RecipeDefinition
    );
    write_schema!(
        &out_dir.join("TechDefinition.json"),
        "TechDefinition",
        TechDefinition
    );
    write_schema!(
        &out_dir.join("ShipDefinition.json"),
        "ShipDefinition",
        ShipDefinition
    );
    write_schema!(
        &out_dir.join("StationDefinition.json"),
        "StationDefinition",
        StationDefinition
    );
    write_schema!(
        &out_dir.join("AuthoredDefaults.json"),
        "AuthoredDefaults",
        AuthoredDefaults
    );
    write_schema!(
        &out_dir.join("GateDefinition.json"),
        "GateDefinition",
        GateDefinition
    );
    write_schema!(
        &out_dir.join("GatePhaseDefinition.json"),
        "GatePhaseDefinition",
        GatePhaseDefinition
    );

    // ── Command DTOs ──
    write_schema!(
        &out_dir.join("CommandEnvelope.json"),
        "CommandEnvelope",
        CommandEnvelope
    );
    write_schema!(
        &out_dir.join("CommandAcknowledgement.json"),
        "CommandAcknowledgement",
        CommandAcknowledgement
    );

    // ── Lifecycle / API DTOs ──
    write_schema!(
        &out_dir.join("ServerStatus.json"),
        "ServerStatus",
        ServerStatus
    );
    write_schema!(
        &out_dir.join("LoadingStatus.json"),
        "LoadingStatus",
        LoadingStatus
    );

    eprintln!("[schema_export] Done — 37 schemas generated.");
}
