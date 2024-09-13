use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use log::{debug, info, trace, warn};
use tokio::sync::Mutex;

use crate::core::{block_in_place, Handle};

pub type CallbackHandle = Handle;

pub trait Callbacks<E>
where
    E: Display + Clone,
{
    /// Adds a new callback to the event handler, which will be triggered when an event is received.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to be registered.
    ///
    /// # Returns
    ///
    /// An `i64` identifier associated with the added callback.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::CoreCallbacks;
    ///
    /// let event_handler = CoreCallbacks::new();
    /// let callback_id = event_handler.add(|event| {
    ///     // Your callback logic here
    /// });
    /// ```
    ///
    /// The `callback_handle` can be used to later remove the callback if needed.
    fn add(&self, callback: CoreCallback<E>) -> CallbackHandle;

    /// Removes a callback from the event handler using its associated identifier.
    ///
    /// # Arguments
    ///
    /// * `callback_handle` - The `i64` identifier of the callback to be removed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::CoreCallbacks;
    ///
    /// let event_handler = CoreCallbacks::new();
    /// let callback_handle = event_handler.add(|event| {
    ///     // Your callback logic here
    /// });
    ///
    /// // Later, if needed, you can remove the callback using its identifier.
    /// event_handler.remove(callback_handle);
    /// ```
    ///
    /// If the provided `callback_handle` does not correspond to any registered callback, this
    /// function should have no effect.
    fn remove(&self, handle: CallbackHandle);
}

/// The callback type which handles callbacks for changes within the Popcorn FX.
/// This is a generic type that can be reused within the [crate::core] package.
pub type CoreCallback<E> = Box<dyn Fn(E) + Send>;

/// The callbacks holder for Popcorn FX events.
/// It contains one or more [CoreCallback] items which can be invoked by one of the services.
///
/// # Example
///
/// ```no_run
/// use popcorn_fx_core::core::{CoreCallback, CoreCallbacks, Callbacks};
///
/// pub type CallbackExample = CoreCallback<CoreEvent>;
/// pub enum CoreEvent {
///     Change
/// }
///
/// let callback: CallbackExample = Box::new(|e| println!("received {:?}", e));
/// let callbacks = CoreCallbacks::<CoreEvent>::default();
///
/// callbacks.add(callback);
/// callbacks.invoke(CoreEvent::Change);
/// ```
#[derive(Clone)]
pub struct CoreCallbacks<E>
where
    E: Display + Clone,
{
    callbacks: Arc<Mutex<Vec<InternalCallbackHolder<E>>>>,
}

impl<E: Display + Clone> CoreCallbacks<E> {
    /// Invoke all callbacks for the given `event`.
    /// Each callback will receive its own owned instance of the `event`.
    pub fn invoke(&self, event: E) {
        let callbacks = self.callbacks.clone();
        let execute = async move {
            let mutex = callbacks.lock().await;

            trace!(
                "Calling a total of {} callbacks for {{{}}}",
                mutex.len(),
                &event
            );
            for internal_callback in mutex.iter() {
                let callback = &internal_callback.callback;
                callback(event.clone());
            }
        };

        block_in_place(execute)
    }
}

impl<E: Display + Clone> Callbacks<E> for CoreCallbacks<E> {
    fn add(&self, callback: CoreCallback<E>) -> CallbackHandle {
        trace!("Registering new callback to CoreCallbacks");
        let handle = Handle::new();
        let mut mutex = block_in_place(self.callbacks.lock());

        mutex.push(InternalCallbackHolder {
            handle: handle.clone(),
            callback,
        });
        debug!(
            "Added new callback for events, new total callbacks {}",
            mutex.len()
        );
        handle
    }

    fn remove(&self, handle: CallbackHandle) {
        trace!("Removing callback from CoreCallbacks");
        let callbacks = self.callbacks.clone();
        let mut mutex = block_in_place(callbacks.lock());
        let position = mutex.iter().position(|e| e.handle == handle);

        if let Some(position) = position {
            mutex.remove(position);
            info!("Removed callback {} from CoreCallbacks", handle);
        } else {
            warn!("Unable to remove callback {}, callback not found", handle);
        }
    }
}

impl<E: Display + Clone> Debug for CoreCallbacks<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mutex = futures::executor::block_on(self.callbacks.lock());
        write!(f, "CoreCallbacks {{callbacks: {}}}", mutex.len())
    }
}

impl<E: Display + Clone> Default for CoreCallbacks<E> {
    fn default() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(vec![])),
        }
    }
}

struct InternalCallbackHolder<E>
where
    E: Display + Clone,
{
    handle: CallbackHandle,
    callback: CoreCallback<E>,
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use derive_more::Display;

    use crate::testing::init_logger;

    use super::*;

    #[derive(Debug, Display, PartialEq, Clone)]
    struct Event {
        value: String,
    }

    #[test]
    fn test_invoke_callbacks() {
        let (tx, rx) = channel();
        let callbacks = CoreCallbacks::<Event>::default();
        let event = Event {
            value: "lorem".to_string(),
        };

        callbacks.add(Box::new(move |e| {
            tx.send(e).unwrap();
        }));
        callbacks.invoke(event.clone());

        let result = rx.recv_timeout(Duration::from_secs(1)).unwrap();

        assert_eq!(event, result)
    }

    #[test]
    fn test_remove() {
        init_logger();
        let callbacks = CoreCallbacks::<Event>::default();

        let id = callbacks.add(Box::new(move |_| {}));
        let e = block_in_place(callbacks.callbacks.lock());
        assert_eq!(1, e.len());
        drop(e);

        callbacks.remove(id);
        let e = block_in_place(callbacks.callbacks.lock());
        assert_eq!(0, e.len());
    }

    #[test]
    fn test_remove_unknown_id() {
        init_logger();
        let callbacks = CoreCallbacks::<Event>::default();

        callbacks.remove(Handle::new());
        let e = block_in_place(callbacks.callbacks.lock());
        assert_eq!(0, e.len());
    }
}
