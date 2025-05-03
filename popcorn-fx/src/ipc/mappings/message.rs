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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_from() {
        let value = 87654;
        let handle = message::Handle {
            handle: value,
            special_fields: Default::default(),
        };
        let expected_result = Handle::from(value);

        let result = Handle::from(&handle);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_handle_proto_from() {
        let value = 12666;
        let handle = Handle::from(value);
        let expected_result = message::Handle {
            handle: value,
            special_fields: Default::default(),
        };

        let result = message::Handle::from(&handle);

        assert_eq!(expected_result, result);
    }
}
