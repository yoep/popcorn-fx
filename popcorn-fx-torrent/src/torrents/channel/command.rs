use std::fmt::Debug;
use std::time::Duration;

use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::{mpsc, oneshot};
use tokio::time;

use crate::torrents::channel::{ChannelError, Result};

const DEFAULT_COMMAND_TIMEOUT_SECONDS: u64 = 60;

/// The command sender is a special channel wrapper which allows for async responses to commands
/// which are sent over a channel, thus allowing for more flexibility than a standard channel.
#[derive(Debug)]
pub struct CommandSender<C, R>
where
    C: Debug,
{
    /// Sender for command instructions.
    command_sender: mpsc::UnboundedSender<CommandInstruction<C, R>>,
    /// The time to wait for a response of a command
    timeout: Duration,
}

impl<C, R> CommandSender<C, R>
where
    C: Debug,
{
    /// Sends a command and waits for a response with a timeout.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to send.
    ///
    /// # Returns
    ///
    /// A result containing the response or a `ChannelError`.
    pub async fn send(&self, command: C) -> Result<R> {
        // create a new oneshot channel for receiving the response
        let (response_sender, response_receiver) = oneshot::channel::<R>();

        // send the command and pass the response sender along
        self.do_internal_send(command, Some(response_sender))?;

        tokio::select! {
            _ = time::sleep(self.timeout) => {
                Err(ChannelError::Timeout(self.timeout.as_millis() as u64))
            },
            response = response_receiver => {
                response.map_err(|e| ChannelError::Failed(e.to_string()))
            }
        }
    }

    /// Sends a fire-and-forget command without waiting for a response.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to send.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn send_void(&self, command: C) -> Result<()> {
        self.do_internal_send(command, None)
    }

    fn do_internal_send(
        &self,
        command: C,
        response_sender: Option<oneshot::Sender<R>>,
    ) -> Result<()> {
        // verify if the sender is already closed before trying to send the command
        if self.command_sender.is_closed() {
            return Err(ChannelError::Closed);
        }

        // create the command instruction
        let command_instruction = CommandInstruction {
            command,
            response_sender,
        };

        trace!("Trying to send {:?}", command_instruction);
        if let Err(e) = self.command_sender.send(command_instruction) {
            return Err(ChannelError::Failed(e.to_string()));
        }

        Ok(())
    }
}

/// Implement the [Clone] trait rather than using the derive macro,
/// as the derive macro requires the generic traits to also implement the [Clone] trait
/// which might not be the case for all commands or responses.
impl<C, R> Clone for CommandSender<C, R>
where
    C: Debug,
{
    fn clone(&self) -> Self {
        Self {
            command_sender: self.command_sender.clone(),
            timeout: self.timeout,
        }
    }
}

#[derive(Debug)]
pub struct CommandReceiver<C, R>
where
    C: Debug,
{
    /// Receiver for command instructions.
    command_receiver: mpsc::UnboundedReceiver<CommandInstruction<C, R>>,
}

impl<C, R> CommandReceiver<C, R>
where
    C: Debug,
{
    /// Receives the next command instruction.
    ///
    /// # Returns
    ///
    /// An `Option` containing the command instruction if available.
    pub async fn recv(&mut self) -> Option<CommandInstruction<C, R>> {
        if let Some(command) = self.command_receiver.recv().await {
            return Some(command);
        }

        debug!("Command receiver is being closed");
        None
    }
}

/// Creates a new command channel for sending and receiving command instructions.
///
/// # Returns
///
/// A tuple containing the command sender and command receiver.
pub fn new_command_channel<C, R>() -> (CommandSender<C, R>, CommandReceiver<C, R>)
where
    C: Debug,
{
    new_command_channel_with_timeout(Duration::from_secs(DEFAULT_COMMAND_TIMEOUT_SECONDS))
}

/// Creates a new command channel for sending and receiving command instructions with a specified timeout.
///
/// # Arguments
///
/// * `timeout` - The duration to wait for a command response before timing out.
///
/// # Returns
///
/// A tuple containing the command sender and command receiver.
pub fn new_command_channel_with_timeout<C, R>(
    timeout: Duration,
) -> (CommandSender<C, R>, CommandReceiver<C, R>)
where
    C: Debug,
{
    let (command_sender, command_receiver) = mpsc::unbounded_channel::<CommandInstruction<C, R>>();
    let sender = CommandSender {
        command_sender,
        timeout,
    };
    let receiver = CommandReceiver { command_receiver };

    (sender, receiver)
}

#[derive(Display)]
#[display(fmt = "{:?}", command)]
pub struct CommandInstruction<C, R>
where
    C: Debug,
{
    /// The command that needs to be executed.
    pub command: C,
    /// The channel for sending the response on.
    response_sender: Option<oneshot::Sender<R>>,
}

impl<C, R> CommandInstruction<C, R>
where
    C: Debug,
{
    /// Gets a reference to the command.
    ///
    /// # Returns
    ///
    /// A reference to the command.
    pub fn command(&self) -> &C {
        &self.command
    }

    /// Sends a response to the command if a response sender exists.
    ///
    /// # Arguments
    ///
    /// * `response` - The response to send.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn respond(&mut self, response: R) -> Result<()> {
        if let Some(sender) = self.response_sender.take() {
            return sender
                .send(response)
                .map_err(|_| ChannelError::Failed("failed to send command response".to_string()));
        }

        warn!("Response has already been sent, ignoring response");
        Ok(())
    }
}

impl<C, R> Debug for CommandInstruction<C, R>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandInstruction")
            .field("command", &self.command)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use tokio::runtime::Runtime;

    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestCommand {
        Foo,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestCommandResponse {
        FooResponse,
    }

    #[test]
    fn test_send_void() {
        init_logger();
        let (sender, receiver) = new_command_channel::<TestCommand, TestCommandResponse>();

        let result = sender.send_void(TestCommand::Foo);

        assert_eq!(Ok(()), result)
    }

    #[test]
    fn test_send() {
        init_logger();
        let runtime = Runtime::new().unwrap();
        let (tx, rx) = channel();
        let (sender, mut receiver) = new_command_channel::<TestCommand, TestCommandResponse>();

        runtime.spawn(async move {
            if let Some(mut command) = receiver.recv().await {
                tx.send(command.command().clone()).unwrap();
                command.respond(TestCommandResponse::FooResponse).unwrap();
            }
        });

        let response = runtime.block_on(sender.send(TestCommand::Foo)).unwrap();
        let command = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(TestCommandResponse::FooResponse, response);
        assert_eq!(TestCommand::Foo, command);
    }

    #[test]
    fn test_send_timeout() {
        init_logger();
        let runtime = Runtime::new().unwrap();
        let timout = Duration::from_millis(50);
        let (sender, mut receiver) =
            new_command_channel_with_timeout::<TestCommand, TestCommandResponse>(timout.clone());

        runtime.spawn(async move {
            if let Some(_) = receiver.recv().await {
                time::sleep(Duration::from_millis(500)).await;
            }
        });

        let result = runtime.block_on(sender.send(TestCommand::Foo));

        assert_eq!(
            ChannelError::Timeout(timout.as_millis() as u64),
            result.unwrap_err()
        );
    }
}
