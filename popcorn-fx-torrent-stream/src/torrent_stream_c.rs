use std::os::raw::c_char;
use std::sync::Arc;

use log::{trace, warn};

use popcorn_fx_core::{from_c_owned, into_c_owned, into_c_string};
use popcorn_fx_core::core::torrents::{TorrentStream, TorrentStreamEvent, TorrentStreamState};

/// The C compatible struct for [TorrentStream].
#[repr(C)]
#[derive(Debug)]
pub struct TorrentStreamC {
    pub url: *const c_char,
    pub ptr: *mut Arc<dyn TorrentStream>,
}

impl TorrentStreamC {
    pub fn stream(&mut self) -> Option<Arc<dyn TorrentStream>> {
        if !self.ptr.is_null() {
            trace!("Reading Arc<dyn TorrentStream> from pointer {:?}", self.ptr);
            let stream = from_c_owned(self.ptr);
            let result = stream.clone();
            self.ptr = into_c_owned(stream);
            Some(result)
        } else {
            warn!("Unable to read the Arc<dyn TorrentStream> pointer, pointer is null");
            None
        }
    }
}

impl From<Arc<dyn TorrentStream>> for TorrentStreamC {
    fn from(value: Arc<dyn TorrentStream>) -> Self {
        TorrentStreamC {
            url: into_c_string(value.url().to_string()),
            ptr: into_c_owned(value),
        }
    }
}

impl Drop for TorrentStreamC {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            trace!("Disposing Arc<dyn TorrentStream>");
            let _ = from_c_owned(self.ptr);
        }
    }
}

/// The C abi compatible torrent stream event.
#[repr(C)]
#[derive(Debug)]
pub enum TorrentStreamEventC {
    StateChanged(TorrentStreamState)
}

impl From<TorrentStreamEvent> for TorrentStreamEventC {
    fn from(value: TorrentStreamEvent) -> Self {
        match value {
            TorrentStreamEvent::StateChanged(e) => TorrentStreamEventC::StateChanged(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stream_event_c_from() {
        let event = TorrentStreamEvent::StateChanged(TorrentStreamState::Stopped);

        let result = TorrentStreamEventC::from(event);

        match result {
            TorrentStreamEventC::StateChanged(state) => assert_eq!(TorrentStreamState::Stopped, state),
            _ => assert!(false, "expected TorrentStreamEventC::StateChanged")
        }
    }
}