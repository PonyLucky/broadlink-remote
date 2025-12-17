use mpris_server::{
    Metadata, PlayerInterface, PlayerProperty, RootInterface, RootProperty, Signal,
};
use std::collections::HashMap;
use zbus::interface;
use crate::state::AppState;
use std::sync::Arc;
use async_trait::async_trait;

pub struct BroadlinkPlayer {
    state: Arc<AppState>,
}

impl BroadlinkPlayer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl RootInterface for BroadlinkPlayer {
    async fn get_all(&self) -> HashMap<RootProperty, PlayerProperty> {
        let mut props = HashMap::new();
        props.insert(RootProperty::CanQuit, PlayerProperty::CanQuit(true));
        props.insert(RootProperty::CanRaise, PlayerProperty::CanRaise(false));
        props.insert(RootProperty::HasTrackList, PlayerProperty::HasTrackList(false));
        props.insert(RootProperty::Identity, PlayerProperty::Identity("Broadlink Remote".to_string()));
        props.insert(RootProperty::SupportedUriSchemes, PlayerProperty::SupportedUriSchemes(vec![]));
        props.insert(RootProperty::SupportedMimeTypes, PlayerProperty::SupportedMimeTypes(vec![]));
        props
    }

    async fn quit(&self) {
        // Handle quit if needed
    }

    async fn raise(&self) {
        // Handle raise if needed
    }
}

#[async_trait]
impl PlayerInterface for BroadlinkPlayer {
    async fn get_all(&self) -> HashMap<PlayerProperty, PlayerProperty> {
        let mut props = HashMap::new();
        props.insert(PlayerProperty::PlaybackStatus, PlayerProperty::PlaybackStatus(mpris_server::PlaybackStatus::Stopped));
        props.insert(PlayerProperty::LoopStatus, PlayerProperty::LoopStatus(mpris_server::LoopStatus::None));
        props.insert(PlayerProperty::Rate, PlayerProperty::Rate(1.0));
        props.insert(PlayerProperty::Shuffle, PlayerProperty::Shuffle(false));
        props.insert(PlayerProperty::Metadata, PlayerProperty::Metadata(Metadata::default()));
        props.insert(PlayerProperty::Volume, PlayerProperty::Volume(1.0));
        props.insert(PlayerProperty::Position, PlayerProperty::Position(0));
        props.insert(PlayerProperty::MinimumRate, PlayerProperty::MinimumRate(1.0));
        props.insert(PlayerProperty::MaximumRate, PlayerProperty::MaximumRate(1.0));
        props.insert(PlayerProperty::CanGoNext, PlayerProperty::CanGoNext(true));
        props.insert(PlayerProperty::CanGoPrevious, PlayerProperty::CanGoPrevious(true));
        props.insert(PlayerProperty::CanPlay, PlayerProperty::CanPlay(true));
        props.insert(PlayerProperty::CanPause, PlayerProperty::CanPause(true));
        props.insert(PlayerProperty::CanSeek, PlayerProperty::CanSeek(false));
        props.insert(PlayerProperty::CanControl, PlayerProperty::CanControl(true));
        props
    }

    async fn next(&self) {
        log::info!("MPRIS: Next");
        // In a real app, find the active device and send "Next" command
    }

    async fn previous(&self) {
        log::info!("MPRIS: Previous");
    }

    async fn pause(&self) {
        log::info!("MPRIS: Pause");
    }

    async fn play_pause(&self) {
        log::info!("MPRIS: PlayPause");
    }

    async fn stop(&self) {
        log::info!("MPRIS: Stop");
    }

    async fn play(&self) {
        log::info!("MPRIS: Play");
    }

    async fn seek(&self, _offset: i64) {}

    async fn set_position(&self, _track_id: String, _position: i64) {}

    async fn open_uri(&self, _uri: String) {}
}

pub struct PlayerController {
    server: mpris_server::Server<BroadlinkPlayer>,
}

impl PlayerController {
    pub async fn update_metadata(&self, title: &str, artist: &str) -> zbus::Result<()> {
        let mut metadata = Metadata::default();
        metadata.title = Some(title.to_string());
        metadata.artists = Some(vec![artist.to_string()]);
        
        self.server.properties_changed(vec![PlayerProperty::Metadata(metadata)]).await
    }

    pub async fn set_playback_status(&self, status: mpris_server::PlaybackStatus) -> zbus::Result<()> {
        self.server.properties_changed(vec![PlayerProperty::PlaybackStatus(status)]).await
    }
}

pub async fn run_mpris(state: Arc<AppState>) -> zbus::Result<()> {
    let player = BroadlinkPlayer::new(state);
    let connection = zbus::Connection::session().await?;
    
    let server = mpris_server::Server::new(&connection, player).await?;
    
    // In a real app, we would store the server/controller in AppState 
    // to allow updating it when remote state changes.
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
