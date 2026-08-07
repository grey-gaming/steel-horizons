//! Steel Horizons engine CLI — authenticated REST API server.
//!
//! ## Authoritative references
//!
//! - ADR-0003 §Local Security — loopback binding, random token, discovery
//! - TDD 02 §Connection and Authentication
//! - TDD 02 §Backpressure and Limits

#![deny(unsafe_code)]
#![forbid(noop_method_call)]

use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;

use clap::Parser;
use steel_horizons_engine::actor::SimulationActor;
use steel_horizons_engine::api::{self, AppState};
use steel_horizons_engine::content::{ContentCatalog, DefinitionsCatalog, StartingScenario};
use steel_horizons_engine::VERSION;
use tokio::net::TcpListener as TokioListener;

// ─── CLI argument parsing ─────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "steel-horizons-engine",
    version = VERSION,
    about = "Steel Horizons deterministic simulation engine"
)]
struct Cli {
    /// Content directory path (default: engine/content/ relative to the binary).
    #[arg(long = "content-dir", default_value = None)]
    content_dir: Option<String>,

    /// Save directory path (default: user data dir / steel-horizons/saves/).
    #[arg(long = "save-dir", default_value = None)]
    save_dir: Option<String>,

    /// Preferred port (default: 4880).
    #[arg(long = "port", default_value = "4880")]
    port: u16,

    /// Disable authentication (development only).
    #[arg(long = "insecure-no-auth", default_value = "false")]
    insecure_no_auth: bool,

    /// Print the version and exit.
    #[arg(long = "version", action = clap::ArgAction::SetTrue)]
    show_version: bool,
}

// ─── Content loading ──────────────────────────────────────────────────

/// Load content from the content directory.
fn load_content(content_dir: &std::path::Path) -> ContentCatalog {
    let definitions_path = content_dir.join("definitions.v1.json");
    let starting_system_path = content_dir.join("starting_system.v1.json");

    let definitions: DefinitionsCatalog = serde_json::from_str(
        &fs::read_to_string(&definitions_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", definitions_path.display(), e)),
    )
    .unwrap_or_else(|e| panic!("Cannot parse {}: {}", definitions_path.display(), e));

    let starting_system: StartingScenario = serde_json::from_str(
        &fs::read_to_string(&starting_system_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", starting_system_path.display(), e)),
    )
    .unwrap_or_else(|e| panic!("Cannot parse {}: {}", starting_system_path.display(), e));

    ContentCatalog {
        definitions,
        starting_system,
    }
}

// ─── Discovery file ───────────────────────────────────────────────────

/// Write the connection discovery file.
fn write_discovery(user_data_dir: &PathBuf, host: &str, port: u16, token: &str, pid: u32) {
    let discovery_path = user_data_dir.join("connection.json");
    let discovery = serde_json::json!({
        "protocol": "v1",
        "host": host,
        "port": port,
        "token": token,
        "pid": pid,
    });
    fs::create_dir_all(user_data_dir).unwrap();
    fs::write(
        &discovery_path,
        serde_json::to_string_pretty(&discovery).unwrap(),
    )
    .unwrap();
    // Set owner-only permissions (macOS/Linux).
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&discovery_path, perms).unwrap();
    }
    eprintln!("[main] Discovery written to {}", discovery_path.display());
}

// ─── Port binding ─────────────────────────────────────────────────────

/// Find a free port starting from `preferred`, trying up to 10 fallbacks.
fn find_port(preferred: u16) -> (u16, TokioListener) {
    let max_tries = 10;
    for offset in 0..max_tries {
        let port = preferred + offset;
        let addr = format!("127.0.0.1:{}", port);
        match TcpListener::bind(&addr) {
            Ok(tcp_listener) => {
                let tokio_listener = TokioListener::from_std(tcp_listener)
                    .unwrap_or_else(|e| panic!("Cannot convert listener: {}", e));
                eprintln!("[main] Bound to port {}", port);
                return (port, tokio_listener);
            }
            Err(e) => {
                if offset < max_tries - 1 {
                    eprintln!("[main] Port {} in use ({}), trying next...", port, e);
                } else {
                    panic!(
                        "Cannot bind any port from {} to {}",
                        preferred,
                        preferred + max_tries - 1
                    );
                }
            }
        }
    }
    unreachable!()
}

// ─── Random token ─────────────────────────────────────────────────────

/// Generate a random session token (base64url, 32 bytes → 43 chars).
fn generate_token() -> String {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf).expect("RNG failure for session token");
    // Simple base64url encoding.
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut token = String::with_capacity(44);
    let mut i = 0;
    while i < 32 {
        let b0 = buf[i];
        let b1 = if i + 1 < 32 { buf[i + 1] } else { 0 };
        let b2 = if i + 2 < 32 { buf[i + 2] } else { 0 };
        let triplet = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        token.push(CHARS[((triplet >> 18) & 0x3F) as usize] as char);
        token.push(CHARS[((triplet >> 12) & 0x3F) as usize] as char);
        if i + 1 < 32 {
            token.push(CHARS[((triplet >> 6) & 0x3F) as usize] as char);
        }
        if i + 2 < 32 {
            token.push(CHARS[(triplet & 0x3F) as usize] as char);
        }
        i += 3;
    }
    token
}

// ─── Main ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if cli.show_version {
        println!("steel-horizons-engine {}", VERSION);
        return;
    }

    // ── Determine content directory ───────────────────────────────────
    let content_dir = match &cli.content_dir {
        Some(dir) => PathBuf::from(dir),
        None => {
            // Default: relative to the binary's location.
            let exe_dir = std::env::current_exe()
                .expect("Cannot get exe path")
                .parent()
                .expect("Cannot get exe parent")
                .to_path_buf();
            // Walk up to find the repo root (has engine/ and content/).
            let mut candidate = exe_dir.clone();
            loop {
                let engine_dir = candidate.join("engine");
                let content_dir = candidate.join("content");
                if engine_dir.is_dir() && content_dir.is_dir() {
                    break content_dir;
                }
                if !candidate.parent().is_some_and(|p| {
                    let root = p.to_path_buf();
                    root.is_dir()
                }) {
                    panic!("Cannot find content directory. Use --content-dir.");
                }
                candidate = candidate.parent().unwrap().to_path_buf();
            }
        }
    };
    eprintln!("[main] Content directory: {}", content_dir.display());

    // ── Load content ──────────────────────────────────────────────────
    let content = load_content(&content_dir);
    eprintln!(
        "[main] Content loaded: {} recipes, {} techs, {} ships, {} stations",
        content.definitions.recipes.len(),
        content.definitions.technologies.len(),
        content.definitions.ships.len(),
        content.definitions.stations.len(),
    );

    // ── Create actor ──────────────────────────────────────────────────
    let (mut actor, mailbox_tx, snapshot_rx, status_rx) = SimulationActor::new(content);
    let content_arc = actor.content.clone();

    // ── Autoload (NewGame default scenario) ───────────────────────────
    {
        use steel_horizons_engine::actor::ActorMessage;
        use steel_horizons_engine::lifecycle::LoadingOperation;
        use tokio::sync::oneshot;

        let (tx, rx) = oneshot::channel();
        mailbox_tx
            .send(ActorMessage::LoadGame {
                operation: LoadingOperation::NewGame,
                response_tx: tx,
            })
            .unwrap();
        let result = rx.await.unwrap();
        eprintln!(
            "[main] Autoload: lifecycle={:?}, tick={}, error={:?}",
            result.lifecycle, result.tick, result.error
        );
    }

    // ── Port binding and token ────────────────────────────────────────
    let port = cli.port;
    let (actual_port, listener) = find_port(port);
    let token = if cli.insecure_no_auth {
        eprintln!("[main] WARNING: running without authentication (--insecure-no-auth)");
        "insecure-dev-token".to_string()
    } else {
        let t = generate_token();
        eprintln!("[main] Session token (first 4 chars): {}...", &t[..4]);
        t
    };

    // ── Write discovery file ──────────────────────────────────────────
    let user_data_dir = match &cli.save_dir {
        Some(dir) => PathBuf::from(dir),
        None => {
            let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp/steel-horizons"));
            base.join("steel-horizons")
        }
    };
    let pid = std::process::id();
    write_discovery(&user_data_dir, "127.0.0.1", actual_port, &token, pid);

    // ── Build API router ──────────────────────────────────────────────
    let state = AppState {
        mailbox_tx,
        snapshot_rx,
        status_rx,
        token: token.clone(),
        content: content_arc,
    };
    let router = api::build_router(state);

    // ── Start actor and server ────────────────────────────────────────
    let actor_handle = tokio::spawn(async move {
        actor.run().await;
    });

    let server = axum::serve(listener, router).with_graceful_shutdown(async {
        let ctrl_c = tokio::signal::ctrl_c();
        ctrl_c.await.unwrap();
        eprintln!("[main] Shutting down...");
    });

    server.await.unwrap_or_else(|e| {
        panic!("Server error: {}", e);
    });
    actor_handle.await.unwrap_or_else(|e| {
        panic!("Actor error: {:?}", e);
    });
    eprintln!("[main] Done.");
}
