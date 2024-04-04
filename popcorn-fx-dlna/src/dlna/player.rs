use std::sync::Weak;

use async_trait::async_trait;
use derive_more::Display;
use rupnp::{Device, Service};
use tokio::sync::Mutex;

use popcorn_fx_core::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};
use popcorn_fx_core::core::players::{Player, PlayerEvent, PlayerState, PlayRequest};

const DLNA_GRAPHIC_RESOURCE: &[u8] = include_bytes!("../../resources/external-dlna-icon.png");

#[derive(Debug, Display)]
#[display(fmt = "{}", id)]
pub struct DlnaPlayer {
    id: String,
    device: Device,
    service: Service,
    state: Mutex<PlayerState>,
    callbacks: CoreCallbacks<PlayerEvent>,
}

impl DlnaPlayer {
    pub fn new(device: Device, service: Service) -> Self {
        let id = format!(
            "[{}]{}",
            device.device_type(),
            device.friendly_name());
        Self {
            id,
            device,
            service,
            state: Mutex::new(PlayerState::Ready),
            callbacks: Default::default(),
        }
    }
}

impl Callbacks<PlayerEvent> for DlnaPlayer {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

#[async_trait]
impl Player for DlnaPlayer {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn name(&self) -> &str {
        self.device.friendly_name()
    }

    fn description(&self) -> &str {
        "DLNA Player"
    }

    fn graphic_resource(&self) -> Vec<u8> {
        DLNA_GRAPHIC_RESOURCE.to_vec()
    }

    fn state(&self) -> PlayerState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        todo!()
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        todo!()
    }

    fn pause(&self) {
        todo!()
    }

    fn resume(&self) {
        todo!()
    }

    fn seek(&self, time: u64) {
        todo!()
    }

    fn stop(&self) {
        todo!()
    }
}