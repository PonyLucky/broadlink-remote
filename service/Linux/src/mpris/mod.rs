use mpris_server::{
    Metadata, PlayerInterface, RootInterface, PlaybackStatus, LoopStatus, Time, Property, TrackId
};
use crate::state::AppState;
use std::sync::Arc;

pub struct BroadlinkPlayer {
    state: Arc<AppState>,
}

impl BroadlinkPlayer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
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
        Ok(())
    }

    async fn previous(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Previous");
        Ok(())
    }

    async fn pause(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Pause");
        Ok(())
    }

    async fn play_pause(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: PlayPause");
        Ok(())
    }

    async fn stop(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Stop");
        Ok(())
    }

    async fn play(&self) -> Result<(), zbus::fdo::Error> {
        log::info!("MPRIS: Play");
        Ok(())
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
        Ok(true)
    }

    async fn can_go_previous(&self) -> Result<bool, zbus::fdo::Error> {
        Ok(true)
    }

    async fn can_play(&self) -> Result<bool, zbus::fdo::Error> {
        Ok(true)
    }

    async fn can_pause(&self) -> Result<bool, zbus::fdo::Error> {
        Ok(true)
    }

    async fn can_seek(&self) -> Result<bool, zbus::fdo::Error> {
        Ok(false)
    }

    async fn can_control(&self) -> Result<bool, zbus::fdo::Error> {
        Ok(true)
    }
}

pub struct PlayerController {
    server: mpris_server::Server<BroadlinkPlayer>,
}

impl PlayerController {
    pub async fn update_metadata(&self, title: &str, artist: &str) -> zbus::Result<()> {
        let mut metadata = Metadata::new();
        metadata.set_title(Some(title.to_string()));
        metadata.set_artist(Some(vec![artist.to_string()]));
        
        self.server.properties_changed(vec![Property::Metadata(metadata)]).await
    }

    pub async fn set_playback_status(&self, status: PlaybackStatus) -> zbus::Result<()> {
        self.server.properties_changed(vec![Property::PlaybackStatus(status)]).await
    }
}

pub async fn run_mpris(state: Arc<AppState>) -> zbus::Result<()> {
    let player = BroadlinkPlayer::new(state);
    
    let _server = mpris_server::Server::new("broadlink_remote", player).await?;
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
