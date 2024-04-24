use std::process::{Command, Stdio};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, info, trace};
use tokio::sync::Mutex;

use popcorn_fx_core::core::block_in_place;
use popcorn_fx_core::core::players::PlayerManager;
use popcorn_fx_core::core::subtitles::{SubtitleManager, SubtitleProvider};

use crate::{Discovery, DiscoveryError, DiscoveryState};
use crate::vlc::VlcPlayer;

/// VLC discovery service responsible for searching and registering an external VLC player.
#[derive(Debug, Display)]
#[display(fmt = "VLC local player discovery")]
pub struct VlcDiscovery {
    subtitle_manager: Arc<Box<dyn SubtitleManager>>,
    subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
    player_manager: Arc<Box<dyn PlayerManager>>,
    state: Mutex<DiscoveryState>,
}

impl VlcDiscovery {
    /// Creates a new instance of `VlcDiscovery`.
    pub fn new(
        subtitle_manager: Arc<Box<dyn SubtitleManager>>,
        subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
        player_manager: Arc<Box<dyn PlayerManager>>,
    ) -> Self {
        Self {
            subtitle_manager,
            subtitle_provider,
            player_manager,
            state: Mutex::new(DiscoveryState::Stopped),
        }
    }

    async fn update_state_async(&self, state: DiscoveryState) {
        let mut mutex = self.state.lock().await;
        debug!("Updating VLC discovery state to {:?}", state);
        *mutex = state.clone();
        info!("VLC discovery state changed to {:?}", state);
    }

    #[cfg(target_os = "windows")]
    fn command() -> Command {
        Command::new("where.exe")
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn command() -> Command {
        Command::new("which")
    }
}

#[async_trait]
impl Discovery for VlcDiscovery {
    fn state(&self) -> DiscoveryState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
    }

    async fn start_discovery(&self) -> crate::Result<()> {
        let state: DiscoveryState;

        {
            let mutex = self.state.lock().await;
            state = mutex.clone();
        }

        if state != DiscoveryState::Running {
            trace!("Searching for external VLC executable");
            self.update_state_async(DiscoveryState::Running).await;
            if Self::command()
                .arg("vlc")
                .stdout(Stdio::null())
                .status()
                .map(|e| e.success())
                .unwrap_or(false)
            {
                trace!("Creating new external VLC player instance");
                let vlc_player = VlcPlayer::builder()
                    .subtitle_manager(self.subtitle_manager.clone())
                    .subtitle_provider(self.subtitle_provider.clone())
                    .build();
                debug!("Created new external VLC player {:?}", vlc_player);
                if self.player_manager.add_player(Box::new(vlc_player)) {
                    info!("Added new external VLC player");
                } else {
                    self.update_state_async(DiscoveryState::Error).await;
                    return Err(DiscoveryError::Initialization(
                        "Unable to add external VLC player".to_string(),
                    ));
                }
            } else {
                info!("External VLC executable not found, external VLC player won't be registered");
            }

            self.update_state_async(DiscoveryState::Stopped).await;
        } else {
            return Err(DiscoveryError::InvalidState(state));
        }

        Ok(())
    }

    fn stop_discovery(&self) -> crate::Result<()> {
        // no-op
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use popcorn_fx_core::core::block_in_place;
    use popcorn_fx_core::core::players::MockPlayerManager;
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::testing::{init_logger, MockSubtitleManager};

    use crate::vlc::VLC_ID;

    use super::*;

    #[test]
    fn test_start_discovery() {
        init_logger();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let (tx, rx) = channel();
        let mut player_manager = MockPlayerManager::new();
        player_manager
            .expect_add_player()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
                true
            });
        let discovery = VlcDiscovery::new(
            Arc::new(Box::new(manager)),
            Arc::new(Box::new(provider)),
            Arc::new(Box::new(player_manager)),
        );

        block_in_place(discovery.start_discovery()).unwrap();

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(VLC_ID, result.id());
    }

    #[test]
    fn test_stop_discovery() {
        init_logger();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player_manager = MockPlayerManager::new();
        let discovery = VlcDiscovery::new(
            Arc::new(Box::new(manager)),
            Arc::new(Box::new(provider)),
            Arc::new(Box::new(player_manager)),
        );

        let result = discovery.stop_discovery();

        assert_eq!(Ok(()), result);
    }
}
