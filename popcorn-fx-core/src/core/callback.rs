use crate::core::CallbackHandle;
use log::{debug, trace};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// The subscription type for the interested event.
/// Drop this subscription to remove the callback.
pub type Subscription<T> = UnboundedReceiver<Arc<T>>;

/// The subscriber type for the interested event.
/// This can be used to send the interested event from multiple sources into one receiver.
pub type Subscriber<T> = UnboundedSender<Arc<T>>;

/// Allows adding callbacks to the struct.
/// The struct will inform the [Subscription] when a certain event occurs.
///
/// # Example
///
/// ```rust,no_run
///
/// use std::sync::Arc;
/// use tokio::runtime::Runtime;
/// use popcorn_fx_core::core::callback::{Callback, MultiCallback};
///
/// #[derive(Debug)]
/// pub enum MyEvent {
///     Foo,
///     Bar,
/// }
///
/// async fn register_callback(shared_runtime: Arc<Runtime>) {
///     let callback = MultiCallback::<MyEvent>::new(shared_runtime);
///     let mut receiver = callback.subscribe();
///
///     let event = receiver.recv().await.unwrap();
///     // do something with the event
/// }
/// ```
pub trait Callback<T>: Debug
where
    T: Debug + Send + Sync,
{
    /// Subscribe to the interested event.
    /// This creates a new [Subscription] that will be invoked with a shared instance of the event when the interested event occurs.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    ///
    /// use popcorn_fx_core::core::callback::Callback;
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// pub enum MyEvent {
    ///     Foo,
    /// }
    ///
    /// async fn example(callback: &dyn Callback<MyEvent>) {
    ///     let mut receiver = callback.subscribe();
    ///     
    ///     if let Some(event) = receiver.recv().await {
    ///         // do something with the event
    ///     }
    /// }
    ///
    /// ```
    ///
    /// # Returns
    ///
    /// It returns a [Subscription] which can be dropped to remove the callback.
    fn subscribe(&self) -> Subscription<T>;

    /// Subscribe to the interested event with a [Subscriber].
    /// This creates an underlying new subscription which will be invoked with the given subscriber when the interested event occurs.
    ///
    /// ## Remarks
    ///
    /// It is possible to grant multiple subscriptions from the same source to the same interested event,
    /// as the [Callback] is only a holder for the [Subscription] and can't detect any duplicates.
    fn subscribe_with(&self, subscriber: Subscriber<T>);
}

/// A multithreaded callback holder.
///
/// This callback holder will invoke the given events on a separate thread, thus unblocking the caller thread for other tasks.
#[derive(Debug)]
pub struct MultiCallback<T>
where
    T: Debug + Send + Sync,
{
    base: Arc<BaseCallback<T>>,
}

impl<T> Callback<T> for MultiCallback<T>
where
    T: Debug + Send + Sync,
{
    fn subscribe(&self) -> Subscription<T> {
        let mut mutex = self.base.callbacks.lock().expect("failed to acquire lock");
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = CallbackHandle::new();
        mutex.insert(handle, tx);
        drop(mutex);
        trace!("Added callback {} to {:?}", handle, self);
        rx
    }

    fn subscribe_with(&self, subscriber: Subscriber<T>) {
        let mut mutex = self.base.callbacks.lock().expect("failed to acquire lock");
        let handle = CallbackHandle::new();
        mutex.insert(handle, subscriber);
        drop(mutex);
        trace!("Added callback {} to {:?}", handle, self);
    }
}

impl<T> MultiCallback<T>
where
    T: Debug + Send + Sync + 'static,
{
    /// Creates a new multithreaded callback.
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            base: Arc::new(BaseCallback::<T>::new(runtime)),
        }
    }

    /// Invokes the callback with the given value.
    pub fn invoke(&self, value: T) {
        let inner = self.base.clone();
        self.base.runtime.spawn(async move {
            let mut mutex = inner.callbacks.lock().expect("failed to acquire lock");
            let mut handles_to_remove = Vec::with_capacity(0);
            let value = Arc::new(value);

            for (handle, callback) in mutex.iter() {
                if let Err(_) = callback.send(value.clone()) {
                    trace!("Callback {} has been dropped", handle);
                    handles_to_remove.push(handle.clone());
                }
            }

            let total_handles = handles_to_remove.len();
            for handle in handles_to_remove {
                mutex.remove(&handle);
            }

            if total_handles > 0 {
                debug!("Removed a total of {} callbacks", total_handles);
            }
        });
    }
}

struct BaseCallback<T>
where
    T: Debug + Send + Sync,
{
    callbacks: Mutex<HashMap<CallbackHandle, UnboundedSender<Arc<T>>>>,
    runtime: Arc<Runtime>,
}

impl<T> BaseCallback<T>
where
    T: Debug + Send + Sync,
{
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            callbacks: Mutex::new(HashMap::new()),
            runtime,
        }
    }
}

impl<T> Debug for BaseCallback<T>
where
    T: Debug + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseCallback")
            .field("callbacks", &self.callbacks.lock().unwrap().len())
            .field("runtime", &self.runtime)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Event {
        Foo,
    }

    #[test]
    fn test_invoke() {
        init_logger!();
        let runtime = Arc::new(Runtime::new().unwrap());
        let expected_result = Event::Foo;
        let (tx, rx) = channel();
        let callback = MultiCallback::<Event>::new(runtime.clone());

        let mut receiver = callback.subscribe();
        runtime.spawn(async move {
            if let Some(e) = receiver.recv().await {
                tx.send(e).unwrap();
            }
        });

        callback.invoke(expected_result.clone());
        let result = rx.recv_timeout(Duration::from_millis(50)).unwrap();

        assert_eq!(expected_result, *result);
    }

    #[test]
    fn test_invoke_dropped_receiver() {
        init_logger!();
        let runtime = Arc::new(Runtime::new().unwrap());
        let expected_result = Event::Foo;
        let (tx, rx) = channel();
        let callback = MultiCallback::<Event>::new(runtime.clone());

        let _ = callback.subscribe();
        let mut receiver = callback.subscribe();
        runtime.spawn(async move {
            if let Some(e) = receiver.recv().await {
                tx.send(e).unwrap();
            }
        });

        callback.invoke(expected_result.clone());
        let result = rx.recv_timeout(Duration::from_millis(50)).unwrap();

        assert_eq!(expected_result, *result);
    }
}
