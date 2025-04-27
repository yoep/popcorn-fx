use crate::ipc::proto::message;
use fx_handle::Handle;
use popcorn_fx_core::core::loader::LoadingHandle;

impl From<&Handle> for message::Handle {
    fn from(value: &LoadingHandle) -> Self {
        Self {
            handle: value.value(),
            special_fields: Default::default(),
        }
    }
}

impl From<&message::Handle> for LoadingHandle {
    fn from(value: &message::Handle) -> Self {
        Self::from(value.handle)
    }
}
