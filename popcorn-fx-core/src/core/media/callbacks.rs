use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use log::{debug, trace};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

/// The media callback type which handles callbacks for media changes.
/// This is a generic type that can be reused within the [crate::core::media] package.
pub type MediaCallback<E> = Box<dyn Fn(E) + Send>;

/// The callbacks holder for media events.
/// It contains one or more [MediaCallback] items which can be invoked by one of the media services.
///
/// The generic type [E] should be an enum.
pub struct MediaCallbacks<E>
    where E: Display + Clone {
    callbacks: Arc<Mutex<Vec<MediaCallback<E>>>>,
}

impl<E: Display + Clone> MediaCallbacks<E> {
    pub fn add(&self, callback: MediaCallback<E>) {
        trace!("Registering new callback for event");
        match Handle::try_current() {
            Ok(e) => {
                e.block_on(self.add_async(callback));
            }
            Err(_) => {
                // let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to be created");
                futures::executor::block_on(self.add_async(callback));
            }
        }
    }

    pub async fn add_async(&self, callback: MediaCallback<E>) {
        let callbacks = self.callbacks.clone();
        let mut mutex = callbacks.lock().await;

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

impl<E: Display + Clone> Debug for MediaCallbacks<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mutex = futures::executor::block_on(self.callbacks.lock());
        write!(f, "MediaCallbacks {{callbacks: {}}}", mutex.len())
    }
}

impl<E: Display + Clone> Default for MediaCallbacks<E> {
    fn default() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(vec![]))
        }
    }
}