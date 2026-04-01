use futures::FutureExt;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{io, result};
use tokio::sync::{mpsc, oneshot};

/// The result type of channel operations.
pub type Result<T> = result::Result<T, io::Error>;

/// The channel sending half for sending messages between tasks.
#[derive(Debug)]
pub struct ChannelSender<T> {
    inner: InnerSenderChannel<T>,
}

impl<T> ChannelSender<T> {
    /// Returns `true` if the channel has been closed.
    /// In this case, the channel will no longer accept any messages.
    pub fn is_closed(&self) -> bool {
        self.inner.0.is_closed()
    }

    /// Send the given message closure to the channel.
    ///
    /// The `M` message mapper accepts a reply sender to send the result of the channel operation.
    pub async fn send<M, R, E, S>(&self, message: M) -> Response<R, E>
    where
        M: FnOnce(Reply<S>) -> T,
        Response<R, E>: From<oneshot::Receiver<S>>,
    {
        let (tx, rx) = oneshot::channel::<S>();
        match self.inner.send(message(Reply::new(tx))).await {
            Ok(()) => rx.into(),
            Err(_) => Response::closed(),
        }
    }

    /// Send the given message to the channel without waiting for a response.
    pub async fn fire_and_forget(&self, message: T) {
        let _ = self.inner.send(message).await;
    }
}

impl<T> Clone for ChannelSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// The channel receiver half for sending messages between tasks.
#[derive(Debug)]
pub struct ChannelReceiver<T> {
    inner: InnerReceiverChannel<T>,
}

impl<T> ChannelReceiver<T> {
    /// Receives the next value from the channel.
    pub async fn recv(&mut self) -> Option<T> {
        self.inner.recv().await
    }

    /// Receives the next value from the channel, returning immediately if no value is available.
    pub fn try_recv(&mut self) -> Option<T> {
        self.inner.recv().now_or_never().flatten()
    }
}

/// Receives a value from the channel and returns a result.
///
/// # Example
///
/// Create a response from a value receiver.
/// The value type will always result in a `Result<T, ChannelError>`.
///
/// ```rust,no_run
/// use tokio::sync::oneshot;
///
/// let (tx, rx) = oneshot::channel::<bool>();
/// Response::from(rx).await; // returns Ok(true) or Err(ChannelError)
/// ```
///
/// Create a response from a value result receiver.
///
/// ```rust,no_run
/// use tokio::sync::oneshot;
///
/// enum CustomError {
///     Parse,
///     Closed,
/// }
///
/// impl From<ChannelError> for CustomError {
///     fn from(_: ChannelError) -> Self {
///         Self::Closed
///     }
/// }
///
/// let (tx, rx) = oneshot::channel::<Result<bool, CustomError>>();
/// Response::from(rx).await; // returns Ok(true) or Err(CustomError)
/// ```
#[derive(Debug)]
pub struct Response<T, E> {
    inner: InnerResponse<T, E>,
}

impl<T, E> Response<T, E> {
    /// Create a failed response which immediately returns as an error.
    pub fn err(e: E) -> Self {
        Self {
            inner: InnerResponse::Err(Some(e)),
        }
    }

    /// Create a closed response which immediately returns as an closed error.
    pub fn closed() -> Self {
        Self {
            inner: InnerResponse::Closed,
        }
    }
}

impl<T, E> Future for Response<T, E>
where
    E: Unpin,
    io::Error: Into<E>,
{
    type Output = result::Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(cx)
    }
}

impl<T> From<oneshot::Receiver<T>> for Response<T, io::Error> {
    fn from(rx: oneshot::Receiver<T>) -> Self {
        Self {
            inner: InnerResponse::PendingMapper(rx),
        }
    }
}

impl<T, E> From<oneshot::Receiver<result::Result<T, E>>> for Response<T, E> {
    fn from(rx: oneshot::Receiver<result::Result<T, E>>) -> Self {
        Self {
            inner: InnerResponse::Pending(rx),
        }
    }
}

enum InnerResponse<T, E> {
    Pending(oneshot::Receiver<result::Result<T, E>>),
    PendingMapper(oneshot::Receiver<T>),
    Err(Option<E>),
    Closed,
}

impl<T, E> InnerResponse<T, E> {
    fn closed() -> io::Error {
        io::Error::new(io::ErrorKind::Interrupted, "channel closed")
    }
}

impl<T, E> Future for InnerResponse<T, E>
where
    E: Unpin,
    io::Error: Into<E>,
{
    type Output = result::Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this {
            InnerResponse::Pending(rx) => Pin::new(rx)
                .poll(cx)
                .map(|res| res.unwrap_or_else(|_| Err(Self::closed().into()))),
            InnerResponse::PendingMapper(rx) => Pin::new(rx).poll(cx).map(|res| match res {
                Ok(v) => Ok(v),
                Err(_) => Err(Self::closed().into()),
            }),
            InnerResponse::Err(e) => {
                let err = e.take().unwrap_or_else(|| Self::closed().into());
                *this = InnerResponse::Closed;
                Poll::Ready(Err(err))
            }
            InnerResponse::Closed => Poll::Ready(Err(Self::closed().into())),
        }
    }
}

impl<T, E> Debug for InnerResponse<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending(_) => write!(f, "Pending"),
            Self::PendingMapper(_) => write!(f, "PendingMapper"),
            Self::Err(_) => write!(f, "Err"),
            Self::Closed => write!(f, "Closed"),
        }
    }
}

/// Reply to a channel request with a value.
#[derive(Debug)]
pub struct Reply<T> {
    inner: InnerReply<T>,
}

impl<T> Reply<T> {
    fn new(inner: oneshot::Sender<T>) -> Self {
        Self {
            inner: InnerReply::Sender(inner),
        }
    }

    /// Create an empty reply channel.
    pub fn empty() -> Self {
        Self {
            inner: InnerReply::Empty,
        }
    }

    /// Send the given value as a response to the channel request.
    pub fn send(self, value: T) {
        match self.inner {
            InnerReply::Sender(inner) => {
                let _ = inner.send(value);
            }
            InnerReply::Empty => {}
        }
    }

    /// Take the inner resolution sender.
    pub(crate) fn take(self) -> oneshot::Sender<T> {
        match self.inner {
            InnerReply::Sender(e) => e,
            InnerReply::Empty => oneshot::channel().0,
        }
    }
}

#[derive(Debug)]
enum InnerReply<T> {
    Sender(oneshot::Sender<T>),
    Empty,
}

#[derive(Debug)]
struct InnerSenderChannel<T>(mpsc::Sender<T>);

impl<T> InnerSenderChannel<T> {
    async fn send(&self, value: T) -> Result<()> {
        self.0.send(value).await.map_err(|_| Self::closed())
    }

    fn closed() -> io::Error {
        io::Error::new(io::ErrorKind::Interrupted, "channel closed")
    }
}

impl<T> Clone for InnerSenderChannel<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug)]
struct InnerReceiverChannel<T>(mpsc::Receiver<T>);

impl<T> InnerReceiverChannel<T> {
    async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }
}

/// Create a new channel for sending and receiving messages between tasks.
///
/// This macro supports:
/// - `channel!()` for a bounded (backpressure) channel with default capacity `256`.
/// - `channel!(N)` for a bounded (backpressure) channel with capacity `N`.
macro_rules! channel {
    () => {{
        crate::channel!(256)
    }};
    ($limit:expr) => {{
        let limit: usize = $limit;
        crate::core::channel::channel(limit)
    }};
}

/// Create a new backpressure channel for sending and receiving messages between tasks.
pub fn channel<T>(limit: usize) -> (ChannelSender<T>, ChannelReceiver<T>) {
    let (sender, receiver) = mpsc::channel(limit);
    (
        ChannelSender {
            inner: InnerSenderChannel(sender),
        },
        ChannelReceiver {
            inner: InnerReceiverChannel(receiver),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::{select, time};

    #[derive(Debug)]
    enum TestCommand {
        RequestWithArgAndResponse { arg: u32, response: Reply<bool> },
        FireAndForget { tx: oneshot::Sender<()> },
        None,
    }

    mod bounded {
        use super::*;

        #[tokio::test]
        async fn test_send_and_receive() {
            let arg = 42;
            let (sender, receiver) = channel!(2);

            let response = sender
                .send(|tx| TestCommand::RequestWithArgAndResponse { arg, response: tx })
                .await;

            validate_response(arg, response, receiver).await;
        }

        #[tokio::test]
        async fn test_fire_and_forget() {
            let (sender, receiver) = channel!(2);
            let (tx, rx) = oneshot::channel();
            start_receiver_processor(receiver);

            sender
                .fire_and_forget(TestCommand::FireAndForget { tx })
                .await;

            select! {
                _ = time::sleep(Duration::from_millis(250)) => assert!(false, "expected the fire and forget to have been processed"),
                _ = rx => {},
            }
        }

        #[tokio::test]
        async fn test_backpressure() {
            let (sender, mut receiver) = channel!(1);

            // send the first message, which is never processed
            sender.fire_and_forget(TestCommand::None).await;

            // try to send a second message, which should be blocked until the first message has been processed
            let future = sender.fire_and_forget(TestCommand::None);
            tokio::pin!(future);
            select! {
                _ = time::sleep(Duration::from_millis(50)) => {},
                _ = &mut future => assert!(false, "expected the second message to be blocked"),
            }

            // process the first message to unblock the second message
            receiver
                .recv()
                .await
                .expect("expected to receive the first message");

            // try again to send the second message
            select! {
                _ = time::sleep(Duration::from_millis(100)) => assert!(false, "expected the second message to be processed"),
                _ = &mut future => {},
            }
        }
    }

    mod reply {
        use super::*;

        #[tokio::test]
        async fn test_send_sender() {
            let expected_result = 42;
            let (tx, rx) = oneshot::channel();
            let reply = Reply::new(tx);

            reply.send(expected_result);

            let result = timeout!(rx, Duration::from_millis(100)).expect("expected a reply");
            assert_eq!(expected_result, result);
        }

        #[tokio::test]
        async fn test_send_empty() {
            let reply = Reply::empty();

            reply.send(66);

            // no-op
        }

        #[tokio::test]
        async fn test_take_empty() {
            let reply = Reply::<u32>::empty();

            assert_eq!(true, reply.take().is_closed());
        }
    }

    fn start_receiver_processor(mut receiver: ChannelReceiver<TestCommand>) {
        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    TestCommand::FireAndForget { tx } => tx.send(()).unwrap(),
                    _ => {}
                }
            }
        });
    }

    async fn validate_response<E: Debug + From<io::Error> + Unpin>(
        expected_arg_value: u32,
        response: Response<bool, E>,
        mut receiver: ChannelReceiver<TestCommand>,
    ) {
        let result = receiver.recv().await;
        if let Some(TestCommand::RequestWithArgAndResponse {
            arg: received_arg,
            response,
        }) = result
        {
            assert_eq!(
                received_arg, expected_arg_value,
                "expected the message argument to match"
            );
            response.send(true);
        } else {
            assert!(false, "expected TestCommand::Lorem, but got {:?}", result);
        }

        let result = response.await.expect("expected a response");
        assert_eq!(true, result, "expected the response to be true");
    }
}
