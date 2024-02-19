use std::process::{Command, Stdio};
use std::sync::Arc;

use log::{debug, info, trace, warn};

use popcorn_fx_core::core::players::PlayerManager;
use popcorn_fx_core::core::subtitles::{SubtitleManager, SubtitleProvider};

use crate::vlc::VlcPlayer;

/// VLC discovery service responsible for searching and registering an external VLC player.
#[derive(Debug)]
pub struct VlcDiscovery {
    subtitle_manager: Arc<Box<dyn SubtitleManager>>,
    subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
    player_manager: Arc<Box<dyn PlayerManager>>,
}

impl VlcDiscovery {
    /// Creates a new instance of `VlcDiscovery`.
    pub fn new(subtitle_manager: Arc<Box<dyn SubtitleManager>>,
               subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
               player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        Self {
            subtitle_manager,
            subtitle_provider,
            player_manager,
        }
    }

    /// Starts the VLC discovery process.
    pub async fn start(&self) {
        trace!("Searching for external VLC executable");
        if Self::command()
            .arg("vlc")
            .stdout(Stdio::null())
            .status()
            .map(|e| e.success())
            .unwrap_or(false) {
            trace!("Creating new external VLC player instance");
            let vlc_player = VlcPlayer::builder()
                .subtitle_manager(self.subtitle_manager.clone())
                .subtitle_provider(self.subtitle_provider.clone())
                .build();
            debug!("Created new external VLC player {:?}", vlc_player);
            if self.player_manager.add_player(Box::new(vlc_player)) {
                info!("Added new external VLC player");
            } else {
                warn!("Unable to add external VLC player");
            }
        } else {
            info!("External VLC executable not found, external VLC player won't be registered");
        }
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
    fn test_start() {
        init_logger();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let (tx, rx) = channel();
        let mut player_manager = MockPlayerManager::new();
        player_manager.expect_add_player()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
                true
            });
        let discovery = VlcDiscovery::new(Arc::new(Box::new(manager)), Arc::new(Box::new(provider)), Arc::new(Box::new(player_manager)));

        block_in_place(discovery.start());

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(VLC_ID, result.id());
    }
}