mod api_client;
mod mpris;
mod tray;
mod state;

use std::sync::Arc;
use tokio::runtime::Runtime;
use crate::state::AppState;
use crate::tray::BroadlinkTray;
use ksni::Handle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    log::info!("Starting Broadlink Remote Linux Port");

    // Load configuration (in a real app, from a file or env)
    let host = std::env::var("BROADLINK_HOST").unwrap_or_else(|_| "192.168.1.143".to_string());
    let port = std::env::var("BROADLINK_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(6676);

    // Initialize state
    let state = Arc::new(AppState::new(host, port));

    // Initial refresh
    let initial_state = state.clone();
    tokio::spawn(async move {
        initial_state.refresh_devices().await;
        log::info!("Initial device refresh complete");
    });

    // Start MPRIS server
    let mpris_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = mpris::run_mpris(mpris_state).await {
            log::error!("MPRIS error: {}", e);
        }
    });

    // Start System Tray
    let tray = BroadlinkTray::new(state.clone());
    let tray_handle = Handle::new(tray);
    tray_handle.spawn();

    log::info!("Broadlink Remote is running");

    // Keep the main task alive
    tokio::signal::ctrl_c().await?;
    log::info!("Shutting down");

    Ok(())
}
