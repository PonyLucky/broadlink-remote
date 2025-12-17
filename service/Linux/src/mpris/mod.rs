use mpris_server::{
    Metadata, PlayerInterface, RootInterface, PlaybackStatus, LoopStatus, Time, TrackId
};
use crate::state::AppState;
use crate::config::MprisCommands;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct BroadlinkPlayer {
    state: Arc<AppState>,
    handle: Handle,
}

impl BroadlinkPlayer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { 
            state,
            handle: Handle::current(),
        }
    }

    async fn send_mpris_command(&self, get_cmd: impl FnOnce(&MprisCommands) -> &String) -> Result<(), zbus::fdo::Error> {
        let config = self.state.mpris_config.read().await;
        if !config.enable || config.controller.is_empty() || config.device.is_empty() {
            return Ok(());
        }

        let path = get_cmd(&config.commands).to_string();
        if path.is_empty() {
            return Ok(());
        }

        let controller = config.controller.clone();
        let device = config.device.clone();
        let state = self.state.clone();
        
        // Spawn on Tokio runtime to ensure reactor is available
        self.handle.spawn(async move {
            let actual_path = state.find_command(&controller, &device, &path).await.unwrap_or_else(|| path.clone());
            match state.client.send_command(&controller, &device, &actual_path).await {
                Ok(true) => log::info!("MPRIS: Sent {} to {}/{}", actual_path, controller, device),
                Ok(false) => log::warn!("MPRIS: Failed to send {} to {}/{}", actual_path, controller, device),
                Err(e) => log::error!("MPRIS: Error sending command: {}", e),
            }
        });

        Ok(())
    }
}

impl RootInterface for BroadlinkPlayer {
    async fn can_quit(&self) -> Result<bool, zbus::fdo::Error> { Ok(true) }
    async fn fullscreen(&self) -> Result<bool, zbus::fdo::Error> { Ok(false) }
    async fn set_fullscreen(&self, _fullscreen: bool) -> Result<(), zbus::Error> { Ok(()) }
    async fn can_set_fullscreen(&self) -> Result<bool, zbus::fdo::Error> { Ok(false) }
    async fn can_raise(&self) -> Result<bool, zbus::fdo::Error> { Ok(false) }
    async fn has_track_list(&self) -> Result<bool, zbus::fdo::Error> { Ok(false) }
    async fn identity(&self) -> Result<String, zbus::fdo::Error> { Ok("Broadlink Remote".to_string()) }
    async fn desktop_entry(&self) -> Result<String, zbus::fdo::Error> { Ok("broadlink-remote".to_string()) }
    async fn supported_uri_schemes(&self) -> Result<Vec<String>, zbus::fdo::Error> { Ok(vec![]) }
    async fn supported_mime_types(&self) -> Result<Vec<String>, zbus::fdo::Error> { Ok(vec![]) }

    async fn quit(&self) -> Result<(), zbus::fdo::Error> {
        Ok(())
    }

    async fn raise(&self) -> Result<(), zbus::fdo::Error> {
        Ok(())
    }
}

impl PlayerInterface for BroadlinkPlayer {
    async fn next(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Next");
        self.send_mpris_command(|c| &c.next).await
    }

    async fn previous(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Previous");
        self.send_mpris_command(|c| &c.previous).await
    }

    async fn pause(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Pause");
        self.send_mpris_command(|c| &c.play_pause).await
    }

    async fn play_pause(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: PlayPause");
        self.send_mpris_command(|c| &c.play_pause).await
    }

    async fn stop(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Stop");
        Ok(())
    }

    async fn play(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Play");
        self.send_mpris_command(|c| &c.play_pause).await
    }

    async fn seek(&self, _offset: Time) -> Result<(), zbus::fdo::Error> {
        Ok(())
    }

    async fn set_position(&self, _track_id: TrackId, _position: Time) -> Result<(), zbus::fdo::Error> {
        Ok(())
    }

    async fn open_uri(&self, _uri: String) -> Result<(), zbus::fdo::Error> {
        Ok(())
    }

    async fn playback_status(&self) -> Result<PlaybackStatus, zbus::fdo::Error> {
        Ok(PlaybackStatus::Stopped)
    }

    async fn loop_status(&self) -> Result<LoopStatus, zbus::fdo::Error> {
        Ok(LoopStatus::None)
    }

    async fn set_loop_status(&self, _status: LoopStatus) -> Result<(), zbus::Error> {
        Ok(())
    }

    async fn rate(&self) -> Result<f64, zbus::fdo::Error> {
        Ok(1.0)
    }

    async fn set_rate(&self, _rate: f64) -> Result<(), zbus::Error> {
        Ok(())
    }

    async fn shuffle(&self) -> Result<bool, zbus::fdo::Error> {
        Ok(false)
    }

    async fn set_shuffle(&self, _shuffle: bool) -> Result<(), zbus::Error> {
        Ok(())
    }

    async fn metadata(&self) -> Result<Metadata, zbus::fdo::Error> {
        Ok(Metadata::new())
    }

    async fn volume(&self) -> Result<f64, zbus::fdo::Error> {
        Ok(1.0)
    }

    async fn set_volume(&self, _volume: f64) -> Result<(), zbus::Error> {
        Ok(())
    }

    async fn position(&self) -> Result<Time, zbus::fdo::Error> {
        Ok(Time::from_micros(0))
    }

    async fn minimum_rate(&self) -> Result<f64, zbus::fdo::Error> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> Result<f64, zbus::fdo::Error> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> Result<bool, zbus::fdo::Error> {
        let config = self.state.mpris_config.read().await;
        Ok(config.enable && !config.controller.is_empty() && !config.device.is_empty() && !config.commands.next.is_empty())
    }

    async fn can_go_previous(&self) -> Result<bool, zbus::fdo::Error> {
        let config = self.state.mpris_config.read().await;
        Ok(config.enable && !config.controller.is_empty() && !config.device.is_empty() && !config.commands.previous.is_empty())
    }

    async fn can_play(&self) -> Result<bool, zbus::fdo::Error> {
        let config = self.state.mpris_config.read().await;
        Ok(config.enable && !config.controller.is_empty() && !config.device.is_empty() && !config.commands.play_pause.is_empty())
    }

    async fn can_pause(&self) -> Result<bool, zbus::fdo::Error> {
        let config = self.state.mpris_config.read().await;
        Ok(config.enable && !config.controller.is_empty() && !config.device.is_empty() && !config.commands.play_pause.is_empty())
    }

    async fn can_seek(&self) -> Result<bool, zbus::fdo::Error> {
        Ok(false)
    }

    async fn can_control(&self) -> Result<bool, zbus::fdo::Error> {
        let config = self.state.mpris_config.read().await;
        Ok(config.enable && !config.controller.is_empty() && !config.device.is_empty())
    }
}


pub async fn run_mpris(state: Arc<AppState>) -> zbus::Result<()> {
    let player = BroadlinkPlayer::new(state);
    
    let _server = mpris_server::Server::new("broadlink_remote", player).await?;
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
