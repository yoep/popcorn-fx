use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use log::{debug, trace};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

/// The callback type which handles callbacks for changes within the Popcorn FX.
/// This is a generic type that can be reused within the [crate::core] package.
pub type CoreCallback<E> = Box<dyn Fn(E) + Send>;

/// The callbacks holder for media events.
/// It contains one or more [CoreCallback] items which can be invoked by one of the services.
///
/// The generic type [E] should be an enum.
pub struct CoreCallbacks<E>
    where E: Display + Clone {
    callbacks: Arc<Mutex<Vec<CoreCallback<E>>>>,
}

impl<E: Display + Clone> CoreCallbacks<E> {
    pub fn add(&self, callback: CoreCallback<E>) {
        trace!("Registering new callback for event");
        let callbacks = self.callbacks.clone();
        let mut mutex = callbacks.blocking_lock();

        mutex.push(callback);
        debug!("Added new callback for events, new total callbacks {}", mutex.len());
    }

    pub fn invoke(&self, event: E) {
        let callbacks = self.callbacks.clone();
        let execute = async move {
            let mutex = callbacks.lock().await;

            debug!("Calling a total of {} callbacks for: {}", mutex.len(), &event);
            for callback in mutex.iter() {
                callback(event.clone());
            }
        };

        match Handle::try_current() {
            Ok(e) => e.block_on(execute),
            Err(_) => {
                let runtime = tokio::runtime::Runtime::new().expect("expected a new runtime");
                runtime.block_on(execute)
            }
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
            callbacks: Arc::new(Mutex::new(vec![]))
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use derive_more::Display;

    use super::*;

    #[derive(Debug, Display, PartialEq, Clone)]
    struct Event {
        value: String,
    }

    #[test]
    fn test_invoke_callbacks() {
        let (tx, rx) = channel();
        let callbacks = CoreCallbacks::<Event>::default();
        let event = Event { value: "lorem".to_string() };

        callbacks.add(Box::new(move |e| {
            tx.send(e).unwrap();
        }));
        callbacks.invoke(event.clone());

        let result = rx.recv_timeout(Duration::from_secs(1)).unwrap();

        assert_eq!(event, result)
    }
}