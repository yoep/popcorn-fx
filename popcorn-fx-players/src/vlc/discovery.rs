use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, info, trace, warn};
use tokio::process::Command;
use tokio::sync::Mutex;

use popcorn_fx_core::core::players::PlayerManager;
use popcorn_fx_core::core::subtitles::{SubtitleManager, SubtitleProvider};

use crate::vlc::VlcPlayer;
use crate::{Discovery, DiscoveryError, DiscoveryState};

/// VLC discovery service responsible for searching and registering an external VLC player.
#[derive(Debug, Display)]
#[display("VLC local player discovery")]
pub struct VlcDiscovery {
    subtitle_manager: Arc<Box<dyn SubtitleManager>>,
    subtitle_provider: Arc<dyn SubtitleProvider>,
    player_manager: Arc<Box<dyn PlayerManager>>,
    state: Mutex<DiscoveryState>,
}

impl VlcDiscovery {
    /// Creates a new instance of `VlcDiscovery`.
    pub fn new(
        subtitle_manager: Arc<Box<dyn SubtitleManager>>,
        subtitle_provider: Arc<dyn SubtitleProvider>,
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
    fn which_program() -> &'static str {
        "where.exe"
    }

    #[cfg(target_family = "unix")]
    fn which_program() -> &'static str {
        "which"
    }

    async fn is_vlc_available() -> bool {
        let mut cmd = Command::new(Self::which_program());
        let status = cmd
            .arg("vlc")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        match status {
            Ok(s) => s.success(),
            Err(_) => false,
        }
    }
}

#[async_trait]
impl Discovery for VlcDiscovery {
    async fn state(&self) -> DiscoveryState {
        *self.state.lock().await
    }

    async fn start_discovery(&self) -> crate::Result<()> {
        let current_state = { *self.state.lock().await };
        if current_state == DiscoveryState::Running {
            return Err(DiscoveryError::InvalidState(current_state));
        }

        trace!("Searching for external VLC executable");
        self.update_state_async(DiscoveryState::Running).await;
        if Self::is_vlc_available().await {
            trace!("Creating new external VLC player instance");
            let vlc_player = VlcPlayer::builder()
                .subtitle_manager(self.subtitle_manager.clone())
                .subtitle_provider(self.subtitle_provider.clone())
                .build();
            debug!("Created new external VLC player {:?}", vlc_player);
            if let Err(e) = self.player_manager.add_player(Box::new(vlc_player)) {
                warn!("Failed to register VLC player, {}", e);
                self.update_state_async(DiscoveryState::Error).await;
                return Err(DiscoveryError::Initialization(
                    "Unable to add external VLC player".to_string(),
                ));
            } else {
                info!("Added new external VLC player");
            }
        } else {
            info!("External VLC executable not found, external VLC player won't be registered");
        }

        self.update_state_async(DiscoveryState::Stopped).await;
        Ok(())
    }

    fn stop_discovery(&self) -> crate::Result<()> {
        // no-op
        Ok(())
    }
}

#[cfg(test)]
#[cfg(not(target_os = "windows"))]
mod tests {
    use popcorn_fx_core::core::players::MockPlayerManager;
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::testing::MockSubtitleManager;
    use popcorn_fx_core::{init_logger, recv_timeout};
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    use crate::vlc::VLC_ID;

    use super::*;

    #[tokio::test]
    async fn test_start_discovery() {
        init_logger!();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let (tx, mut rx) = unbounded_channel();
        let mut player_manager = MockPlayerManager::new();
        player_manager
            .expect_add_player()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
                Ok(())
            });
        let discovery = VlcDiscovery::new(
            Arc::new(Box::new(manager)),
            Arc::new(provider),
            Arc::new(Box::new(player_manager)),
        );

        discovery.start_discovery().await.unwrap();

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        assert_eq!(VLC_ID, result.id());
    }

    #[tokio::test]
    async fn test_stop_discovery() {
        init_logger!();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player_manager = MockPlayerManager::new();
        let discovery = VlcDiscovery::new(
            Arc::new(Box::new(manager)),
            Arc::new(provider),
            Arc::new(Box::new(player_manager)),
        );

        let result = discovery.stop_discovery();

        assert_eq!(Ok(()), result);
    }
}
