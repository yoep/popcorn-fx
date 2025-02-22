use crate::core::loader::loading_chain::LoadingChain;
use crate::core::loader::{
    LoadingData, LoadingError, LoadingEvent, LoadingHandle, LoadingResult, LoadingState,
};
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_util::sync::{
    CancellationToken, WaitForCancellationFuture, WaitForCancellationFutureOwned,
};

/// Represents a task responsible for loading media items in a playlist.
///
/// The `LoadingTask` manages loading processes, including handling loading events, canceling loading, and more.
#[derive(Debug, Display)]
#[display(fmt = "{}", context)]
pub struct LoadingTask {
    context: Arc<LoadingTaskContext>,
}

impl LoadingTask {
    /// Creates a new `LoadingTask` instance.
    ///
    /// The task is associated with a loading chain and will manage loading processes for media items in the playlist.
    ///
    /// # Arguments
    ///
    /// * `chain` - An `Arc` to the loading chain containing loading strategies.
    /// * `runtime` - The [Runtime] instance to use for executing the loading task in the background.
    ///
    /// # Returns
    ///
    /// A new `LoadingTask` instance.
    pub fn new(chain: Arc<LoadingChain>, runtime: Arc<Runtime>) -> Self {
        let (event_sender, event_receiver) = unbounded_channel();
        let (command_sender, command_receiver) = unbounded_channel();
        let inner = Arc::new(LoadingTaskContext::new(
            chain,
            event_sender,
            command_sender,
            runtime.clone(),
        ));

        let event_inner = inner.clone();
        runtime.spawn(async move {
            event_inner
                .start(&event_inner, event_receiver, command_receiver)
                .await;
        });

        debug!("Loading task {} created", inner.handle);
        Self { context: inner }
    }

    /// Get an owned instance of the task handle.
    pub fn handle(&self) -> LoadingHandle {
        self.context.handle
    }

    /// Get the task handle as reference.
    pub fn handle_as_ref(&self) -> &LoadingHandle {
        &self.context.handle
    }

    /// Get the current loading state of the task.
    ///
    /// # Returns
    ///
    /// The current loading state.
    pub async fn state(&self) -> LoadingState {
        *self.context.state.lock().await
    }

    /// Start loading the given data.
    /// The load operation is offloaded into the main loop of the task.
    ///
    /// # Arguments
    ///
    /// * `data` - The data that needs to be loaded in the task.
    pub fn load(&self, data: LoadingData) {
        self.context
            .send_command_event(LoadingCommandEvent::Load(data));
    }

    /// Cancels the loading process associated with the task.
    ///
    /// This method cancels the loading process and any ongoing loading operation.
    pub fn cancel(&self) {
        self.context.cancellation_token.cancel();
    }

    /// Get the loading task context.
    pub fn context(&self) -> &Arc<LoadingTaskContext> {
        &self.context
    }
}

impl Callback<LoadingEvent> for LoadingTask {
    fn subscribe(&self) -> Subscription<LoadingEvent> {
        self.context.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<LoadingEvent>) {
        self.context.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for LoadingTask {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[derive(Debug, PartialEq)]
enum LoadingCommandEvent {
    /// Start loading the given data through the chain
    Load(LoadingData),
    /// Indicates that the loading task has ended
    Finished,
}

/// The context information of a loading task.
#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
pub struct LoadingTaskContext {
    /// The unique task handle identifier
    handle: LoadingHandle,
    /// The current state of the loading task
    state: Mutex<LoadingState>,
    /// The chain of tasks that need to be executed
    chain: Arc<LoadingChain>,
    /// The event sender for updating the task while executing the chain
    event_sender: UnboundedSender<LoadingEvent>,
    /// The command event sender of the loading task
    command_sender: UnboundedSender<LoadingCommandEvent>,
    /// The callback of the loading task
    callbacks: MultiThreadedCallback<LoadingEvent>,
    /// The cancellation token of the task
    cancellation_token: CancellationToken,
    /// The shared runtime of the task
    runtime: Arc<Runtime>,
}

impl LoadingTaskContext {
    fn new(
        chain: Arc<LoadingChain>,
        event_sender: UnboundedSender<LoadingEvent>,
        command_sender: UnboundedSender<LoadingCommandEvent>,
        runtime: Arc<Runtime>,
    ) -> Self {
        Self {
            handle: Default::default(),
            state: Mutex::new(LoadingState::Initializing),
            chain,
            event_sender,
            command_sender,
            callbacks: MultiThreadedCallback::new(runtime.clone()),
            cancellation_token: Default::default(),
            runtime,
        }
    }

    /// Check if the task is cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Returns a Future that gets fulfilled when the loading task is cancelled.
    ///
    /// The future will complete immediately if the loading task is already cancelled when this method is called.
    pub fn cancelled(&self) -> WaitForCancellationFuture<'_> {
        self.cancellation_token.cancelled()
    }

    /// Returns a Future that gets fulfilled when the loading task is cancelled.
    ///
    /// The future will complete immediately if the loading task is already cancelled when this method is called.
    pub fn cancelled_owned(&self) -> WaitForCancellationFutureOwned {
        self.cancellation_token.clone().cancelled_owned()
    }

    /// Inform the task about a loading event.
    /// This will send the loading event info to the task subscribers and media loader.
    pub fn send_event(&self, event: LoadingEvent) {
        trace!("Loading task {} is invoking event {}", self, event);
        if let Err(_) = self.event_sender.send(event) {
            debug!("Loading task {} event sender channel has been closed", self);
            self.cancellation_token.cancel();
        }
    }

    /// Get the underlying runtime that is being used for loading the task.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Start the loading task main chain loop.
    async fn start(
        &self,
        context: &Arc<LoadingTaskContext>,
        mut event_receiver: UnboundedReceiver<LoadingEvent>,
        mut command_receiver: UnboundedReceiver<LoadingCommandEvent>,
    ) {
        trace!("Loading task {} is starting", self);

        loop {
            select! {
                event = event_receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_event(event).await;
                    } else {
                        break;
                    }
                }
                command = command_receiver.recv() => {
                    if let Some(event) = command {
                        if self.handle_command_event(event, context).await {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        debug!("Loading task {} main loop ended", self);
    }

    async fn handle_event(&self, event: LoadingEvent) {
        debug!("Loading task {} received event {:?}", self, event);
        if let LoadingEvent::StateChanged(state) = &event {
            self.update_state(state.clone()).await;
        }

        self.callbacks.invoke(event);
    }

    /// Handle the given loading command event.
    /// It returns `true` when the loading task has been completed, else `false`.
    async fn handle_command_event(
        &self,
        event: LoadingCommandEvent,
        context: &Arc<LoadingTaskContext>,
    ) -> bool {
        debug!("Loading task {} received command event {:?}", self, event);
        match event {
            LoadingCommandEvent::Load(data) => {
                let load_context = context.clone();
                self.runtime.spawn(async move {
                    load_context.load(data).await;
                });
                false
            }
            LoadingCommandEvent::Finished => true,
        }
    }

    /// Set the state of the loading task
    async fn update_state(&self, state: LoadingState) {
        let mut mutex = self.state.lock().await;
        if *mutex == state {
            return;
        }

        *mutex = state;
        debug!("Loading task {} state changed to {}", self, state);
        self.callbacks.invoke(LoadingEvent::StateChanged(state));
    }

    async fn load(&self, mut data: LoadingData) {
        let strategies = self.chain.strategies();

        trace!(
            "Loading task {} is processing a total of {} loading strategies",
            self,
            strategies.len(),
        );
        self.callbacks
            .invoke(LoadingEvent::StateChanged(LoadingState::Initializing));
        for strategy in strategies.iter() {
            if let Some(strategy) = strategy.upgrade() {
                trace!("Loading task {} executing {}", self, strategy);
                let start_time = Instant::now();

                select! {
                    _ = self.cancellation_token.cancelled() => break,
                    result = strategy.process(&mut data, &self) => {
                        let elapsed = start_time.elapsed();
                        debug!(
                            "Loading task {} strategy {} executed in {}.{:03}ms",
                            self,
                            strategy,
                            elapsed.as_millis(),
                            elapsed.subsec_micros() % 1000
                        );

                        if self.handle_process_result(result) {
                            break;
                        }
                    }
                }
            } else {
                warn!("Loading task {} strategy is no longer in scope", self);
                break;
            }
        }

        // check if the loading has been cancelled
        // if so, we undo any changes made by the strategies
        if self.cancellation_token.is_cancelled() {
            trace!("Loading task {} is being cancelled", self);
            self.cancel(data).await;
        }
    }

    async fn cancel(&self, mut data: LoadingData) {
        let strategies = self.chain.strategies();

        debug!(
            "Loading task {} is cancelling {} strategies",
            self,
            strategies.len()
        );
        for index in (0..strategies.len()).rev() {
            if let Some(strategy) = strategies.get(index).and_then(|e| e.upgrade()) {
                trace!("Cancelling {}", strategy);
                match strategy.cancel(data).await {
                    Ok(new_data) => data = new_data,
                    Err(e) => {
                        error!(
                            "Loading task {} cancellation of {} failed, {}",
                            self, strategy, e
                        );
                        self.invoke_event(LoadingEvent::LoadingError(e));
                        break;
                    }
                }
            } else {
                warn!(
                    "Loading task {} cancellation failed, strategy went out of scope",
                    self
                );
            }
        }

        info!("Loading task {} has been cancelled", self.handle);
        self.update_state(LoadingState::Cancelled).await;
        self.invoke_event(LoadingEvent::LoadingError(LoadingError::Cancelled));
        self.invoke_event(LoadingEvent::Cancelled);
        self.send_command_event(LoadingCommandEvent::Finished);
    }

    /// Handle the [LoadingResult] of a strategy which has been processed.
    ///
    /// # Returns
    ///
    /// It returns `true` when the loading task chain should be ended.
    fn handle_process_result(&self, result: LoadingResult) -> bool {
        match result {
            LoadingResult::Ok => false,
            LoadingResult::Completed => {
                debug!("Loading task {} strategies have been completed", self);
                self.invoke_event(LoadingEvent::Completed);
                self.send_command_event(LoadingCommandEvent::Finished);
                true
            }
            LoadingResult::Err(err) => {
                if err != LoadingError::Cancelled {
                    error!("Loading task {} encountered an error, {}", self, err);
                    self.invoke_event(LoadingEvent::LoadingError(err));
                    self.send_command_event(LoadingCommandEvent::Finished);
                }
                true
            }
        }
    }

    fn invoke_event(&self, event: LoadingEvent) {
        self.callbacks.invoke(event);
    }

    fn send_command_event(&self, command: LoadingCommandEvent) {
        if let Err(_) = self.command_sender.send(command) {
            debug!(
                "Loading task {} command sender channel has been closed",
                self
            );
            self.cancellation_token.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{channel, Sender};
    use std::time::Duration;

    use async_trait::async_trait;
    use derive_more::Display;
    use tokio::time;

    use super::*;
    use crate::core::block_in_place_runtime;
    use crate::core::loader::LoadingError;
    use crate::core::loader::{CancellationResult, LoadingStrategy, MockLoadingStrategy};
    use crate::core::playlist::PlaylistItem;
    use crate::init_logger;

    #[derive(Debug, Display)]
    #[display(fmt = "CancelStrategy")]
    struct CancelStrategy {
        pub initiated: Sender<()>,
        pub cancelled: Sender<bool>,
    }

    #[async_trait]
    impl LoadingStrategy for CancelStrategy {
        async fn process(
            &self,
            _: &mut LoadingData,
            context: &LoadingTaskContext,
        ) -> LoadingResult {
            self.initiated.send(()).unwrap();

            select! {
                _ = context.cancelled() => {
                    info!("CancelStrategy context is being cancelled");
                    if let Err(e) = self.cancelled.send(true) {
                        error!("Failed to send cancellation result, {}", e);
                    }
                    LoadingResult::Err(LoadingError::Cancelled)
                },
                _ = time::sleep(Duration::from_millis(750)) => LoadingResult::Completed,
            }
        }

        async fn cancel(&self, data: LoadingData) -> CancellationResult {
            Ok(data)
        }
    }

    #[derive(Debug, Display)]
    #[display(fmt = "SleepStrategy")]
    struct SleepStrategy {
        duration: Duration,
    }

    impl SleepStrategy {
        fn new(timeout: Duration) -> Self {
            Self { duration: timeout }
        }
    }

    #[async_trait]
    impl LoadingStrategy for SleepStrategy {
        async fn process(
            &self,
            _data: &mut LoadingData,
            _context: &LoadingTaskContext,
        ) -> LoadingResult {
            time::sleep(self.duration).await;
            LoadingResult::Completed
        }

        async fn cancel(&self, data: LoadingData) -> CancellationResult {
            Ok(data)
        }
    }

    #[test]
    fn test_handle() {
        init_logger!();
        let runtime = Arc::new(Runtime::new().unwrap());
        let task = LoadingTask::new(Arc::new(LoadingChain::from(vec![])), runtime.clone());

        assert_ne!(task.handle().value(), 0i64);
    }

    #[test]
    fn test_state() {
        init_logger!();
        let data = LoadingData::from(PlaylistItem {
            url: None,
            title: "MyStateTest".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let (tx, rx) = channel();
        let mut strategy = MockLoadingStrategy::new();
        strategy
            .expect_process()
            .times(1)
            .returning(move |_, context| {
                context.send_event(LoadingEvent::StateChanged(LoadingState::Downloading));
                block_in_place_runtime(time::sleep(Duration::from_millis(100)), context.runtime());
                LoadingResult::Completed
            });
        let runtime = Arc::new(Runtime::new().unwrap());
        let task = LoadingTask::new(
            Arc::new(LoadingChain::from(vec![
                Box::new(strategy) as Box<dyn LoadingStrategy>
            ])),
            runtime.clone(),
        );

        let mut receiver = task.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    debug!("Received task event {:?}", event);
                    if let LoadingEvent::StateChanged(state) = &*event {
                        // the first event is always initializing, so ignore it
                        if *state != LoadingState::Initializing {
                            tx.send(*state).unwrap();
                        }
                    }
                } else {
                    break;
                }
            }
            debug!("Task event receiver loop ended");
        });

        task.load(data);

        let result = rx.recv_timeout(Duration::from_millis(500)).unwrap();
        assert_eq!(LoadingState::Downloading, result);
    }

    #[test]
    fn test_load() {
        init_logger!();
        let data = LoadingData::from(PlaylistItem {
            url: None,
            title: "MyLoadTest".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let (tx_data, rx_data) = channel();
        let (tx_completed, rx_completed) = channel();
        let mut strategy = MockLoadingStrategy::new();
        strategy
            .expect_process()
            .times(1)
            .returning(move |data, _| {
                tx_data.send(data.clone()).unwrap();
                LoadingResult::Completed
            });
        let runtime = Arc::new(Runtime::new().unwrap());
        let task = LoadingTask::new(
            Arc::new(LoadingChain::from(vec![
                Box::new(strategy) as Box<dyn LoadingStrategy>
            ])),
            runtime.clone(),
        );
        let context = task.context();

        let mut receiver = task.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let LoadingEvent::Completed = &*event {
                        tx_completed.send(()).unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        task.load(data.clone());

        let _ = rx_completed
            .recv_timeout(Duration::from_millis(500))
            .expect("expected the loading task to complete");

        let result = rx_data
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the process to have received data");
        assert_eq!(data, result);
    }

    #[test]
    fn test_cancel_should_return_cancelled_error() {
        init_logger!();
        let data = LoadingData::from(PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let strategy = SleepStrategy::new(Duration::from_secs(50));
        let (tx, rx) = channel();
        let (tx_error, rx_error) = channel();
        let (tx_cancelled, rx_cancelled) = channel();
        let runtime = Arc::new(Runtime::new().unwrap());
        let task = LoadingTask::new(
            Arc::new(LoadingChain::from(vec![
                Box::new(strategy) as Box<dyn LoadingStrategy>
            ])),
            runtime.clone(),
        );

        let mut receiver = task.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    match &*event {
                        LoadingEvent::StateChanged(state) => {
                            if *state == LoadingState::Initializing {
                                tx.send(()).unwrap();
                            }
                        }
                        LoadingEvent::LoadingError(e) => tx_error.send(e.clone()).unwrap(),
                        LoadingEvent::Cancelled => tx_cancelled.send(()).unwrap(),
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        });

        task.load(data);

        let _ = rx
            .recv_timeout(Duration::from_secs(500))
            .expect("expected the task to start");
        task.cancel();

        let result = rx_error
            .recv_timeout(Duration::from_millis(500))
            .expect("expected a LoadingError");
        assert_eq!(
            LoadingError::Cancelled,
            result,
            "expected the cancelled error"
        );

        let _ = rx_cancelled
            .recv_timeout(Duration::from_millis(500))
            .expect("expected a cancelled event");
        let result = runtime.block_on(task.state());
        assert_eq!(LoadingState::Cancelled, result);
    }

    #[test]
    fn test_cancel_should_cancel_strategy_token() {
        init_logger!();
        let data = LoadingData::from(PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let (tx, rx) = channel();
        let (tx_cancelled, rx_cancelled) = channel();
        let strategy = CancelStrategy {
            initiated: tx,
            cancelled: tx_cancelled,
        };
        let runtime = Arc::new(Runtime::new().unwrap());
        let task = LoadingTask::new(
            Arc::new(LoadingChain::from(vec![
                Box::new(strategy) as Box<dyn LoadingStrategy>
            ])),
            runtime.clone(),
        );

        task.load(data);

        let _ = rx
            .recv_timeout(Duration::from_millis(250))
            .expect("expected the strategy process to have been started");
        task.cancel();

        let result = rx_cancelled
            .recv_timeout(Duration::from_millis(750))
            .unwrap();
        assert!(result, "expected the strategy to have been cancelled");
    }

    #[test]
    fn test_cancel_should_call_cancel_when_executed() {
        init_logger!();
        let data = LoadingData::from(PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let (tx, rx) = channel();
        let (tx_cancel, rx_cancel) = channel();
        let mut strat1 = MockLoadingStrategy::new();
        strat1.expect_process().times(1).returning(move |_, _| {
            tx.send(()).unwrap();
            LoadingResult::Ok
        });
        strat1.expect_cancel().times(1).returning(move |e| {
            tx_cancel.send(e.clone()).unwrap();
            Ok(e)
        });
        let strat2 = SleepStrategy::new(Duration::from_millis(200));
        let runtime = Arc::new(Runtime::new().unwrap());
        let task = LoadingTask::new(
            Arc::new(LoadingChain::from(vec![
                Box::new(strat1) as Box<dyn LoadingStrategy>,
                Box::new(strat2) as Box<dyn LoadingStrategy>,
            ])),
            runtime.clone(),
        );

        task.load(data.clone());

        let _ = rx
            .recv_timeout(Duration::from_millis(500))
            .expect("expected the strategy process to have been started");
        task.cancel();

        let result = rx_cancel
            .recv_timeout(Duration::from_millis(500))
            .expect("expected the cancel fn to have been invoked");
        assert_eq!(data, result);
    }

    #[test]
    fn test_loading_task_send_event() {
        init_logger!();
        let expected_event = LoadingEvent::StateChanged(LoadingState::Connecting);
        let runtime = Arc::new(Runtime::new().unwrap());
        let task = LoadingTask::new(Arc::new(LoadingChain::from(vec![])), runtime.clone());
        let context = &task.context;

        let mut receiver = task.subscribe();
        context.send_event(expected_event.clone());

        let result = runtime.block_on(async {
            select! {
                _ = time::sleep(Duration::from_millis(500)) => Err(LoadingError::TimeoutError("event receiver timed out".to_string())),
                Some(event) = receiver.recv() => Ok(event),
            }
        }).unwrap();
        assert_eq!(expected_event, *result);
    }
}
