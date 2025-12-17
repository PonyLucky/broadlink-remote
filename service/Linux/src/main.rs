mod api_client;
mod mpris;
mod tray;
mod state;

use std::sync::{Arc, Mutex};
use crate::state::AppState;
use crate::tray::BroadlinkTray;

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

    // Start MPRIS server
    let mpris_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = mpris::run_mpris(mpris_state).await {
            log::error!("MPRIS error: {}", e);
        }
    });

    // Start System Tray
    let tray_handle_store = Arc::new(Mutex::new(None));
    let tray = BroadlinkTray::new(state.clone(), tray_handle_store.clone());
    let tray_service = ksni::TrayService::new(tray);
    let tray_handle = tray_service.handle();
    
    {
        let mut store = tray_handle_store.lock().unwrap();
        *store = Some(tray_handle.clone());
    }
    
    tray_service.spawn();

    // Initial refresh
    let initial_state = state.clone();
    let initial_tray_handle = tray_handle.clone();
    tokio::spawn(async move {
        initial_state.refresh_devices().await;
        log::info!("Initial device refresh complete");
        initial_tray_handle.update(|_| {});
    });

    log::info!("Broadlink Remote is running");

    // Keep the main task alive
    tokio::signal::ctrl_c().await?;
    log::info!("Shutting down");

    Ok(())
}
