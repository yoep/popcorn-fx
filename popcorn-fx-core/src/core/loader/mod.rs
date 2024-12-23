pub use data::*;
pub use loader_auto_resume::*;
pub use loader_media_torrent::*;
pub use loader_player::*;
pub use loader_subtitles::*;
pub use loader_torrent::*;
pub use loader_torrent_details::*;
pub use loader_torrent_info::*;
pub use loader_torrent_stream::*;
pub use loading_chain::*;
pub use loading_strategy::*;
pub use media_loader::*;

mod data;
mod loader_auto_resume;
mod loader_media_torrent;
mod loader_player;
mod loader_subtitles;
mod loader_torrent;
mod loader_torrent_details;
mod loader_torrent_info;
mod loader_torrent_stream;
mod loading_chain;
mod loading_strategy;
mod media_loader;
mod task;

#[cfg(test)]
pub mod tests {
    use crate::core::loader::task::LoadingTask;
    use crate::core::loader::LoadingChain;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    #[macro_export]
    macro_rules! create_loading_task {
        () => {
            crate::core::loader::tests::new_loading_task(
                std::sync::Arc::new(crate::core::loader::LoadingChain::default()),
                std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()),
            )
        };
        ($chain:expr) => {
            crate::core::loader::tests::new_loading_task(
                $chain,
                std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()),
            )
        };
        ($chain:expr, $runtime:expr) => {
            crate::core::loader::tests::new_loading_task($chain, $runtime)
        };
    }

    pub fn new_loading_task(chain: Arc<LoadingChain>, runtime: Arc<Runtime>) -> LoadingTask {
        LoadingTask::new(chain, runtime)
    }
}
