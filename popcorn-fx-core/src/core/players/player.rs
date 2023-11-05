use std::fmt::{Debug, Display};

pub trait Player: Debug + Display {
    fn id(&self) -> &str;

    fn name(&self) -> &str;

    fn description(&self) -> &str;

    fn graphic_resource(&self) -> Vec<u8>;

    fn state(&self) -> &PlayerState;
}

#[repr(i32)]
#[derive(Debug, Clone)]
pub enum PlayerState {
    Unknown = -1,
    Ready = 0,
    Loading = 1,
    Buffering = 2,
    Playing = 3,
    Paused = 4,
    Stopped = 5,
    Error = 6,
}