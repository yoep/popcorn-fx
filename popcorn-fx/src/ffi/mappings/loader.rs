use popcorn_fx_core::core::loader::{LoaderEvent, LoadingState};

/// A C-compatible callback function type for loader events.
pub type LoaderEventCallback = extern "C" fn(LoaderEventC);

/// A C-compatible enum representing loader events.
#[repr(C)]
#[derive(Debug)]
pub enum LoaderEventC {
    StateChanged(LoadingState),
}

impl From<LoaderEvent> for LoaderEventC {
    fn from(value: LoaderEvent) -> Self {
        match value {
            LoaderEvent::StateChanged(e) => LoaderEventC::StateChanged(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_event_c_from() {
        let state = LoadingState::Downloading;
        let event = LoaderEvent::StateChanged(state.clone());

        let result = LoaderEventC::from(event);

        if let LoaderEventC::StateChanged(result) = result {
            assert_eq!(state, result);
        } else {
            assert!(false, "expected LoaderEventC::StateChanged, but got {:?} instead", result)
        }
    }
}