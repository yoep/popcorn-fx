pub use errors::*;
pub use file_stream::*;
pub use range::*;
pub use resource::*;
pub use server::*;

mod errors;
mod file_stream;
mod media_type;
mod range;
mod resource;
mod server;

#[cfg(any(test, feature = "testing"))]
pub mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::TryStreamExt;
    use fx_callback::{Callback, Subscriber, Subscription};
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub StreamingResource {}

        #[async_trait]
        impl StreamingResource for StreamingResource {
            fn filename(&self) -> &str;
            async fn stream(&self) -> Result<Box<dyn Stream>>;
            async fn stream_range(
                &self,
                start: u64,
                end: Option<u64>,
            ) -> Result<Box<dyn Stream>>;
            async fn state(&self) -> StreamState;
            async fn stop(&self);
        }

        impl Callback<StreamEvent> for StreamingResource {
            fn subscribe(&self) -> Subscription<StreamEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<StreamEvent>);
        }
    }

    /// Reads the stream resource as a string.
    pub async fn read_stream(stream: Box<dyn Stream>) -> String {
        let mut result: Vec<u8> = vec![];
        let mut stream = Box::into_pin(stream);

        while let Ok(Some(data)) = stream.try_next().await {
            result.append(&mut data.to_vec());
        }

        String::from_utf8(result).expect("expected a valid string")
    }
}
