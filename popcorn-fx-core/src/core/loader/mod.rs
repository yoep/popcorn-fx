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
    use crate::core::loader::task::{LoadingTask, LoadingTaskContext};
    use crate::core::loader::{
        CancellationResult, LoadingChain, LoadingData, LoadingError, LoadingEvent, LoadingResult,
        LoadingStrategy,
    };
    use std::fmt::{Debug, Formatter};

    use async_trait::async_trait;
    use derive_more::Display;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::mpsc::UnboundedSender;
    use tokio::{select, time};

    /// Create a new loading task for the given chain of loading strategies
    #[macro_export]
    macro_rules! create_loading_task {
        () => {
            crate::core::loader::tests::new_loading_task(std::sync::Arc::new(
                crate::core::loader::LoadingChain::default(),
            ))
        };
        ($chain:expr) => {
            crate::core::loader::tests::new_loading_task($chain)
        };
    }

    /// Create a new loading task for the given chain of loading strategies
    ///
    /// # Arguments
    ///
    /// * `chain` - The loading chain containing one or more loading strategies.
    ///
    /// # Returns
    ///
    /// It returns a new `LoadingTask` instance.
    pub fn new_loading_task(chain: Arc<LoadingChain>) -> LoadingTask {
        LoadingTask::new(chain)
    }

    /// A loading data peeker that can be used to investigate the triggered process fn.
    pub type DataPeeker = dyn Fn(&LoadingData) + Send + Sync;

    #[derive(Display)]
    #[display("TestingLoadingStrategy")]
    pub struct TestingLoadingStrategy {
        event: Option<LoadingEvent>,
        process_result: LoadingResult,
        delay: Duration,
        data_peeker: Option<Box<DataPeeker>>,
        cancel_sender: Option<UnboundedSender<()>>,
    }

    impl TestingLoadingStrategy {
        pub fn builder() -> TestingLoadingStrategyBuilder {
            TestingLoadingStrategyBuilder::default()
        }

        fn new(
            event: Option<LoadingEvent>,
            process_result: LoadingResult,
            delay: Duration,
            data_peeker: Option<Box<DataPeeker>>,
            cancel_sender: Option<UnboundedSender<()>>,
        ) -> Self {
            Self {
                event,
                process_result,
                delay,
                data_peeker,
                cancel_sender,
            }
        }
    }

    #[async_trait]
    impl LoadingStrategy for TestingLoadingStrategy {
        async fn process(
            &self,
            data: &mut LoadingData,
            context: &LoadingTaskContext,
        ) -> LoadingResult {
            if let Some(peeker) = self.data_peeker.as_ref() {
                peeker(data);
            }
            if let Some(event) = self.event.clone() {
                context.send_event(event);
            }

            select! {
                _ = context.cancelled() => return LoadingResult::Err(LoadingError::Cancelled),
                _ = time::sleep(self.delay) => {}
            }

            self.process_result.clone()
        }

        async fn cancel(&self, _: &mut LoadingData) -> CancellationResult {
            if let Some(sender) = self.cancel_sender.as_ref() {
                sender.send(()).unwrap();
            }
            Ok(())
        }
    }

    impl Debug for TestingLoadingStrategy {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestingLoadingStrategy")
                .field("event", &self.event)
                .field("process_result", &self.process_result)
                .field("delay", &self.delay)
                .field("cancel_sender", &self.cancel_sender)
                .finish()
        }
    }

    #[derive(Default)]
    pub struct TestingLoadingStrategyBuilder {
        event: Option<LoadingEvent>,
        process_result: Option<LoadingResult>,
        delay: Option<Duration>,
        data_peeker: Option<Box<DataPeeker>>,
        cancel_sender: Option<UnboundedSender<()>>,
    }

    impl TestingLoadingStrategyBuilder {
        pub fn event(mut self, event: LoadingEvent) -> Self {
            self.event = Some(event);
            self
        }

        pub fn process_result(mut self, process_result: LoadingResult) -> Self {
            self.process_result = Some(process_result);
            self
        }

        pub fn delay(mut self, delay: Duration) -> Self {
            self.delay = Some(delay);
            self
        }

        pub fn data_peeker<F>(mut self, peeker: F) -> Self
        where
            F: Fn(&LoadingData) + Send + Sync + 'static,
        {
            self.data_peeker = Some(Box::new(peeker) as Box<DataPeeker>);
            self
        }

        pub fn cancel_sender(mut self, sender: UnboundedSender<()>) -> Self {
            self.cancel_sender = Some(sender);
            self
        }

        pub fn build(self) -> TestingLoadingStrategy {
            let process_result = self.process_result.unwrap_or(LoadingResult::Completed);
            let delay = self.delay.unwrap_or(Duration::from_millis(0));

            TestingLoadingStrategy::new(
                self.event,
                process_result,
                delay,
                self.data_peeker,
                self.cancel_sender,
            )
        }
    }
}
