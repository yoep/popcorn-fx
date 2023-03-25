use std::cmp::Ordering;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

use log::{debug, trace};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::core::events::Event;

/// The highest order for events, this priority will be first invoked
pub const HIGHEST_ORDER: Order = i32::MIN;
/// The default order for events, this priority will be first invoked
pub const DEFAULT_ORDER: Order = 0;
/// The lowest order for events, this priority will be last invoked
pub const LOWEST_ORDER: Order = i32::MAX;

/// The event callback type which handles callbacks for events within Popcorn FX.
/// This is a generic type that can be reused within the [crate::core::events] package.
///
/// The callback uses a chain methodology which means it keeps invoking the chain of consumers/listeners for as long as event is being returned.
/// Returning [None] as a result for the callback, will stop the chain from being invoked.
///
/// # Examples
///
/// ## Continue invoking the chain
///
/// ```no_run
/// use popcorn_fx_core::core::events::{Event, EventCallback};
///
/// let continue_consumer: EventCallback = Box::new(|event| {
///     // do something with the event
///     // use the Clone trait if you want to the store the event information
///     Some(event)
/// });
/// ```
///
/// ## Stop the chain
///
/// ```no_run
/// use popcorn_fx_core::core::events::{Event, EventCallback};
///
/// let stop_consumer: EventCallback = Box::new(|event| {
///     // consume the event and prevent the chain from continuing
///     None
/// });
/// ```
pub type EventCallback = Box<dyn Fn(Event) -> Option<Event> + Send>;

/// The event ordering priority type in which the event consumers/listeners will be invoked.
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
/// use popcorn_fx_core::core::events::{Event, EventPublisher};
/// let publisher = EventPublisher::default();
///
/// publisher.publish(Event::PlayerStopped(x));
/// ```
///
/// ## Register consumer/listener
///
/// ```no_run
/// use popcorn_fx_core::core::events::{Event, EventPublisher, HIGHEST_ORDER};
/// let publisher = EventPublisher::default();
///
/// publisher.register(Box::new(|event|Some(event)), HIGHEST_ORDER);
/// ```
pub struct EventPublisher {
    /// The callbacks that need to be invoked for the listener
    callbacks: Arc<Mutex<Vec<EventCallbackHolder>>>,
    runtime: Runtime,
}

impl EventPublisher {
    /// Register a new event consumer/listener to the [EventPublisher].
    pub fn register(&self, callback: EventCallback, order: Order) {
        trace!("Registering a new callback to the EventPublisher");
        let callbacks = self.callbacks.clone();
        let mut mutex = callbacks.blocking_lock();

        mutex.push(EventCallbackHolder {
            order,
            callback,
        });
        mutex.sort();
        debug!("Added event callback, new total callbacks {}", mutex.len());
    }

    /// Publish a new application event.
    pub fn publish(&self, event: Event) {
        let callbacks = self.callbacks.clone();
        self.runtime.spawn(async move {
            let invocations = callbacks.lock().await;
            let mut arg = event;

            trace!("Invoking a total of {} callbacks for the event publisher", invocations.len());
            for invocation in invocations.iter() {
                if let Some(event) = (invocation.callback)(arg) {
                    arg = event;
                } else {
                    debug!("Event publisher chain has been interrupted");
                    break;
                }
            }
        });
    }
}

impl Default for EventPublisher {
    fn default() -> Self {
        Self {
            callbacks: Arc::new(Default::default()),
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .thread_name("events")
                .build()
                .expect("expected a new runtime"),
        }
    }
}

impl Debug for EventPublisher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mutex = &self.callbacks.blocking_lock();
        f.debug_struct("EventPublisher")
            .field("callbacks", &mutex.len())
            .finish()
    }
}

/// The holder is responsible for storing the ordering information of callbacks.
/// It will order the callbacks based on the [Order] value.
struct EventCallbackHolder {
    pub order: Order,
    pub callback: EventCallback,
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
        self.partial_cmp(other).expect("expected an Ordering to be returned")
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::events::{PlayerStoppedEvent, PlayVideoEvent};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_event_publisher_register() {
        init_logger();
        let publisher = EventPublisher::default();

        // Register a new event consumer
        let callback = Box::new(|e| Some(e));
        publisher.register(callback, DEFAULT_ORDER);

        // Check if the event consumer is registered
        let callbacks = publisher.callbacks.blocking_lock();
        assert_eq!(callbacks.len(), 1);
    }

    #[test]
    fn test_event_publisher_publish() {
        init_logger();
        let (tx, rx) = channel();
        let event = PlayerStoppedEvent {
            url: "http://localhost/video.mkv".to_string(),
            media: None,
            time: Some(140000),
            duration: Some(2000000),
        };
        let publisher = EventPublisher::default();

        // Register a new event consumer that handles PlayerStopped events
        let callback: EventCallback = Box::new(move |event| {
            match &event {
                Event::PlayerStopped(stopped_event) => tx.send(stopped_event.clone()).unwrap(),
                _ => {}
            }
            // return the original event to continue the event chain
            Some(event)
        });
        publisher.register(callback, DEFAULT_ORDER);

        // Publish a new PlayerStopped event
        publisher.publish(Event::PlayerStopped(event.clone()));
        let event_result = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        // Check if the event consumer is invoked with the correct event
        assert_eq!(event, event_result)
    }

    #[test]
    fn test_event_publisher_publish_multiple_consumers() {
        init_logger();
        let (tx_callback1, rx_callback1) = channel();
        let (tx_callback2, rx_callback2) = channel();
        let publisher = EventPublisher::default();

        // Register two event consumers that handle PlayerStarted and PlayerStopped events, respectively
        let callback1: EventCallback = Box::new(move |event| {
            match &event {
                Event::PlayerStopped(event) => tx_callback1.send(event.clone()).unwrap(),
                _ => {}
            }
            Some(event)
        });
        let callback2: EventCallback = Box::new(move |event| {
            match &event {
                Event::PlayerStopped(event) => tx_callback2.send(event.clone()).unwrap(),
                _ => {}
            }
            Some(event)
        });
        publisher.register(callback1, LOWEST_ORDER);
        publisher.register(callback2, HIGHEST_ORDER);

        // Publish a new PlayerStopped event
        let event = PlayerStoppedEvent {
            url: "http://localhost/video.mkv".to_string(),
            media: None,
            time: Some(140000),
            duration: Some(2000000),
        };
        publisher.publish(Event::PlayerStopped(event.clone()));

        // Check if the event consumers are invoked in the correct order
        let callback1_result = rx_callback1.recv_timeout(Duration::from_millis(100)).unwrap();
        let callback2_result = rx_callback2.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(event, callback1_result);
        assert_eq!(event, callback2_result);
    }

    #[test]
    fn test_event_publisher_publish_event_consumed() {
        init_logger();
        let (tx_callback1, rx_callback1) = channel();
        let (tx_callback2, rx_callback2) = channel();
        let publisher = EventPublisher::default();

        // Register two event consumers that handle PlayerStarted and PlayerStopped events, respectively
        let callback1: EventCallback = Box::new(move |event| {
            match &event {
                Event::PlayVideo(event) => tx_callback1.send(event.clone()).unwrap(),
                _ => {}
            }
            Some(event)
        });
        let callback2: EventCallback = Box::new(move |event| {
            match &event {
                Event::PlayVideo(event) => tx_callback2.send(event.clone()).unwrap(),
                _ => {}
            }
            None
        });
        publisher.register(callback1, LOWEST_ORDER);
        publisher.register(callback2, HIGHEST_ORDER);

        // Publish a new PlayerStopped event
        let event = PlayVideoEvent {
            url: "http://localhost/video.mkv".to_string(),
            title: "Lorem ipsum".to_string(),
            thumb: None,
        };
        publisher.publish(Event::PlayVideo(event.clone()));

        // Check if the event consumers are invoked in the correct order
        let callback2_result = rx_callback2.recv_timeout(Duration::from_millis(100)).unwrap();
        let callback1_result = rx_callback1.recv_timeout(Duration::from_millis(50));
        assert_eq!(event, callback2_result);
        assert!(callback1_result.is_err(), "expected the rx_callback1 to not have been invoked");
    }
}
