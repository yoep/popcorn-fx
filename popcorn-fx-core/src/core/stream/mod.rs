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
    use fx_callback::{Callback, Subscription};
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub StreamingResource {}

        #[async_trait]
        impl StreamingResource for StreamingResource {
            fn filename(&self) -> &str;
            async fn stream(&self) -> Result<FxStream>;
            async fn stream_range(
                &self,
                start: u64,
                end: Option<u64>,
            ) -> Result<FxStream>;
            async fn state(&self) -> StreamState;
            async fn stop(&self);
        }

        impl Callback<StreamEvent> for StreamingResource {
            fn subscribe(&self) -> Subscription<StreamEvent>;
        }
    }

    /// Reads the stream resource as a string.
    pub async fn read_stream(stream: FxStream) -> String {
        let mut result: Vec<u8> = vec![];
        let mut stream = Box::pin(stream);

        while let Ok(Some(data)) = stream.try_next().await {
            result.append(&mut data.to_vec());
        }

        String::from_utf8(result).expect("expected a valid string")
    }
}
