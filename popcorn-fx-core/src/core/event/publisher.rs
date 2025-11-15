use crate::core::event::{Error, Event, Result};

use fx_handle::Handle;
use log::{debug, error, info, trace, warn};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{oneshot, Mutex};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

/// The highest order for events, this priority will be first invoked
pub const HIGHEST_ORDER: Order = i32::MIN;
/// The default order for events, this priority will be first invoked
pub const DEFAULT_ORDER: Order = 0;
/// The lowest order for events, this priority will be last invoked
pub const LOWEST_ORDER: Order = i32::MAX;

/// The event callback unique identifier.
type EventCallbackHandle = Handle;
/// The event for registering a new event callback.
type RegistrationEvent = (UnboundedSender<EventHandler>, Order);

/// The event callback receiver for events published to the event chain.
pub type EventCallback = UnboundedReceiver<EventHandler>;

#[derive(Debug)]
pub struct EventHandler {
    event: Option<Event>,
    response: Option<oneshot::Sender<Option<Event>>>,
}

impl EventHandler {
    fn new(event: Event) -> (Self, oneshot::Receiver<Option<Event>>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                event: Some(event),
                response: Some(tx),
            },
            rx,
        )
    }

    /// Get the reference to the event that was published.
    pub fn event_ref(&self) -> Option<&Event> {
        self.event.as_ref()
    }

    /// Get the event that was published, consuming the event from the handler.
    pub fn take(&mut self) -> Option<Event> {
        self.event.take()
    }

    /// Continue with the next callback in the event chain.
    /// This allows the publisher to continue or stop processing the event chain.
    pub fn next(&mut self) {
        let event = self.event.take();
        self.next_with(event);
    }

    /// Stop the event chain by consuming the event.
    /// This will make the event publisher stop processing the event chain.
    pub fn stop(&mut self) {
        let _ = self.event.take();
        self.next_with(None);
    }

    /// Continue with the next callback in the event chain with the given event.
    /// This allows the publisher to continue or stop processing the event chain.
    fn next_with(&mut self, event: Option<Event>) {
        if let Some(response) = self.response.take() {
            let _ = response.send(event);
        }
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.next();
    }
}

/// The event ordering priority type that determines the order in which the event consumers/listeners will be invoked.
pub type Order = i32;

/// The event publisher allows for the publishing and listing to application wide events.
/// It decouples components by allowing a central system to handle events to which each component can subscribe without needing the requirement
/// to know who the original publisher is of the event.
///
/// # Examples
///
/// ## Publish a new event
///
/// ```no_run
/// use popcorn_fx_core::core::event::{Event, EventPublisher, PlayerStoppedEvent};
/// let publisher = EventPublisher::default();
///
/// publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
///     url: "".to_string(),
///     media: None,
///     time: None,
///     duration: None,
/// }));
/// ```
///
/// ## Register consumer/listener
///
/// ```no_run
/// use popcorn_fx_core::core::event::{Event, EventPublisher, HIGHEST_ORDER};
/// let publisher = EventPublisher::default();
///
/// let callback = publisher.subscribe(HIGHEST_ORDER);
/// ```
#[derive(Debug, Clone)]
pub struct EventPublisher {
    inner: Arc<InnerEventPublisher>,
}

impl EventPublisher {
    /// Create a new event publisher instance.
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel();
        let inner = Arc::new(InnerEventPublisher {
            sender,
            callbacks: Default::default(),
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(receiver).await;
        });

        Self { inner }
    }

    /// Create a new event subscription with the `EventPublisher`.
    /// This receiver of the subscription will receive all events published to the `EventPublisher`.
    ///
    /// # Arguments
    ///
    /// * `order` - The ordering priority for receiving events. Lower values indicate higher priority.
    ///
    /// # Returns
    ///
    /// It returns the event receiver when the publisher has not yet been closed, else [Error::Closed].
    ///
    /// # Examples
    ///
    /// Registering a new event callback with the highest order:
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::event;
    /// use popcorn_fx_core::core::event::{Event, EventPublisher, EventCallback, Order};
    ///
    /// async fn example(event_publisher: EventPublisher) {
    ///     let mut callback = event_publisher.subscribe(event::HIGHEST_ORDER).unwrap();
    ///     let event = callback.recv().await;
    /// }
    /// ```
    pub fn subscribe(&self, order: Order) -> Result<EventCallback> {
        if self.inner.cancellation_token.is_cancelled() {
            return Err(Error::Closed);
        }

        let (sender, receiver) = unbounded_channel();
        let _ = self
            .inner
            .sender
            .send(EventPublisherCommand::Registration((sender, order)));

        Ok(receiver)
    }

    /// Publish a new application event.
    ///
    /// This method asynchronously invokes the registered event callbacks with the provided event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish.
    pub fn publish(&self, event: Event) {
        let _ = self.inner.sender.send(EventPublisherCommand::Event(event));
    }

    /// Close the event publisher from publishing any new events.
    /// This will terminate the event loop.
    pub fn close(&self) {
        self.inner.cancellation_token.cancel()
    }
}

impl Default for EventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

enum EventPublisherCommand {
    Registration(RegistrationEvent),
    Event(Event),
}

#[derive(Debug)]
struct InnerEventPublisher {
    sender: UnboundedSender<EventPublisherCommand>,
    callbacks: Mutex<Vec<EventCallbackHolder>>,
    cancellation_token: CancellationToken,
}

impl InnerEventPublisher {
    /// Start the main internal loop of the event publisher.
    /// This loop will handle every published event until it is cancelled.
    async fn start(&self, mut receiver: UnboundedReceiver<EventPublisherCommand>) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = receiver.recv() => self.handle_command_event(command).await,
            }
        }

        debug!("Event publisher main loop ended");
    }

    async fn handle_command_event(&self, command: EventPublisherCommand) {
        match command {
            EventPublisherCommand::Registration((sender, order)) => {
                self.handle_callback_registration(sender, order).await
            }
            EventPublisherCommand::Event(event) => self.handle_event(event).await,
        }
    }

    async fn handle_event(&self, event: Event) {
        let invocations = self.callbacks.lock().await;
        let mut invocations_to_remove = vec![];
        info!("Publishing event {}", event);
        let mut event = event;

        debug!(
            "Invoking a total of {} callbacks for the event publisher",
            invocations.len()
        );
        trace!("Invoking callbacks {:?}", invocations);
        for invocation in invocations.iter() {
            let event_info = event.to_string();
            let (event_handler, receiver) = EventHandler::new(event);
            if let Err(mut e) = invocation.sender.send(event_handler) {
                event = e.0.take().expect("expected the event to still be present");
                invocations_to_remove.push(invocation.handle);
                continue;
            }

            select! {
                _ = time::sleep(Duration::from_secs(60)) => {
                    error!("Event publisher callback invocation timed out for {:?}", event_info);
                    break;
                }
                result = receiver => {
                    match result {
                        Ok(result) => {
                            match result {
                                None => {
                                    debug!("Event publisher chain has been interrupted");
                                    break;
                                }
                                Some(result) => event = result,
                            }
                        },
                        Err(_) => {
                            warn!("Event publisher callback invocation failed, response channel closed");
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn handle_callback_registration(
        &self,
        sender: UnboundedSender<EventHandler>,
        order: Order,
    ) {
        trace!("Registering a new callback to the EventPublisher");
        let mut callbacks = self.callbacks.lock().await;
        callbacks.push(EventCallbackHolder {
            handle: Default::default(),
            order,
            sender,
        });
        callbacks.sort();
        debug!(
            "Added event callback, new total callbacks {}",
            callbacks.len()
        );
    }
}

/// The holder is responsible for storing the ordering information of callbacks.
/// It will order the callbacks based on the [Order] value.
struct EventCallbackHolder {
    handle: EventCallbackHandle,
    order: Order,
    sender: UnboundedSender<EventHandler>,
}

impl PartialEq for EventCallbackHolder {
    fn eq(&self, other: &Self) -> bool {
        self.order.eq(&other.order)
    }
}

impl Eq for EventCallbackHolder {}

impl PartialOrd for EventCallbackHolder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for EventCallbackHolder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("expected an Ordering to be returned")
    }
}

impl Debug for EventCallbackHolder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventCallbackHolder")
            .field("order", &self.order)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use crate::core::event::PlayerStoppedEvent;
    use crate::{assert_timeout, init_logger, recv_timeout};

    use std::time::Duration;
    use tokio::time;

    use super::*;

    #[tokio::test]
    async fn test_event_publisher_register() {
        init_logger!();
        let publisher = EventPublisher::default();

        // Register a new event consumer
        let _callback = publisher
            .subscribe(DEFAULT_ORDER)
            .expect("expected to receive a callback receiver");

        // Check if the event consumer is registered
        let callbacks = &publisher.inner.callbacks;
        assert_timeout!(
            Duration::from_millis(200),
            callbacks.lock().await.len() == 1,
            "expected the callback to have been registered"
        );
    }

    #[tokio::test]
    async fn test_event_publisher_register_closed() {
        init_logger!();
        let publisher = EventPublisher::default();

        // close the publisher
        publisher.close();

        // try to register a new callback receiver
        let result = publisher.subscribe(DEFAULT_ORDER);
        if let Err(result) = result {
            assert_eq!(
                Error::Closed,
                result,
                "expected the publisher to have been closed"
            );
        } else {
            assert!(false, "expected Err, but got {:?} instead", result);
        }
    }

    #[tokio::test]
    async fn test_event_publisher_publish() {
        init_logger!();
        let (tx, mut rx) = unbounded_channel();
        let event = PlayerStoppedEvent {
            url: "http://localhost/video.mkv".to_string(),
            media: None,
            time: Some(140000),
            duration: Some(2000000),
        };
        let publisher = EventPublisher::default();

        // Register a new event consumer that handles PlayerStopped events
        let mut callback = publisher.subscribe(DEFAULT_ORDER).unwrap();
        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback.recv().await {
                    if let Some(Event::PlayerStopped(stopped_event)) = handler.event_ref() {
                        tx.send(stopped_event.clone()).unwrap();
                    }

                    // return the original event to continue the event chain
                    handler.next();
                } else {
                    break;
                }
            }
        });

        // Publish a new PlayerStopped event
        publisher.publish(Event::PlayerStopped(event.clone()));
        let event_result = recv_timeout!(&mut rx, Duration::from_millis(100));

        // Check if the event consumer is invoked with the correct event
        assert_eq!(event, event_result)
    }

    #[tokio::test]
    async fn test_event_publisher_publish_multiple_consumers() {
        init_logger!();
        let (tx_callback1, mut rx_callback1) = unbounded_channel();
        let (tx_callback2, mut rx_callback2) = unbounded_channel();
        let publisher = EventPublisher::default();

        // Register two event consumers that handle PlayerStarted and PlayerStopped events, respectively
        let mut callback1 = publisher.subscribe(LOWEST_ORDER).unwrap();
        let mut callback2 = publisher.subscribe(HIGHEST_ORDER).unwrap();

        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback1.recv().await {
                    if let Some(Event::PlayerStopped(event)) = handler.event_ref() {
                        tx_callback1.send(event.clone()).unwrap();
                    }

                    handler.next();
                } else {
                    break;
                }
            }
        });
        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback2.recv().await {
                    if let Some(Event::PlayerStopped(event)) = handler.event_ref() {
                        tx_callback2.send(event.clone()).unwrap();
                    }

                    handler.next();
                } else {
                    break;
                }
            }
        });

        // Publish a new PlayerStopped event
        let event = create_player_stopped_event();
        publisher.publish(Event::PlayerStopped(event.clone()));

        // Check if the event consumers are invoked in the correct order
        let callback1_result = recv_timeout!(&mut rx_callback1, Duration::from_millis(100));
        let callback2_result = recv_timeout!(&mut rx_callback2, Duration::from_millis(100));
        assert_eq!(event, callback1_result);
        assert_eq!(event, callback2_result);
    }

    #[tokio::test]
    async fn test_event_publisher_publish_event_consumed() {
        init_logger!();
        let (tx_callback1, mut rx_callback1) = unbounded_channel();
        let (tx_callback2, mut rx_callback2) = unbounded_channel();
        let publisher = EventPublisher::default();

        // Register two event consumers that handle PlayerStarted and PlayerStopped events, respectively
        let mut callback1 = publisher.subscribe(LOWEST_ORDER).unwrap();
        let mut callback2 = publisher.subscribe(HIGHEST_ORDER).unwrap();

        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback1.recv().await {
                    if let Some(Event::PlayerStopped(event)) = handler.event_ref() {
                        tx_callback1.send(event.clone()).unwrap();
                    }

                    handler.next();
                } else {
                    break;
                }
            }
        });
        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback2.recv().await {
                    if let Some(Event::PlayerStopped(event)) = handler.event_ref() {
                        tx_callback2.send(event.clone()).unwrap();
                    }

                    handler.stop();
                } else {
                    break;
                }
            }
        });

        // Publish a new PlayerStopped event
        let event = create_player_stopped_event();
        publisher.publish(Event::PlayerStopped(event.clone()));

        // Check if the event consumers are invoked in the correct order
        let callback2_result = recv_timeout!(&mut rx_callback2, Duration::from_millis(200));
        assert_eq!(event, callback2_result);
        let callback1_result = select! {
            _ = time::sleep(Duration::from_millis(100)) => false,
            _ = rx_callback1.recv() => true
        };
        assert_eq!(
            false, callback1_result,
            "expected the rx_callback1 to not have been invoked"
        );
    }

    #[tokio::test]
    async fn test_close() {
        init_logger!();
        let (tx, mut rx) = unbounded_channel();
        let publisher = EventPublisher::default();

        // Register a new event consumer that handles PlayerStopped events
        let mut callback = publisher.subscribe(DEFAULT_ORDER).unwrap();
        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback.recv().await {
                    tx.send(
                        handler
                            .take()
                            .expect("expected the event to be present within the handler"),
                    )
                    .unwrap();
                    handler.stop();
                } else {
                    break;
                }
            }
        });

        // close the event publisher
        publisher.close();
        let closed = select! {
            _ = time::sleep(Duration::from_millis(200)) => false,
            _ = publisher.inner.sender.closed() => true,
        };
        assert_eq!(true, closed, "expected the event publisher to be closed");

        // publish a new event after closure
        publisher.publish(Event::PlayerStopped(create_player_stopped_event()));

        let event_result = select! {
            _ = time::sleep(Duration::from_millis(100)) => false,
            Some(_) = rx.recv() => true
        };
        assert_eq!(
            false, event_result,
            "expected the event consumer to not have been invoked after closure"
        );
    }

    fn create_player_stopped_event() -> PlayerStoppedEvent {
        PlayerStoppedEvent {
            url: "https::/localhost:8457/my_video.mkv".to_string(),
            media: None,
            time: Some(10000),
            duration: Some(15000),
        }
    }
}
