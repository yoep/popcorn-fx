use crate::popcorn::fx::platform::platform::Platform;

pub struct PlatformMac {

}

impl PlatformMac {
    pub fn new() -> PlatformMac {
        return PlatformMac {}
    }
}

impl Platform for PlatformMac {
    fn disable_screensaver(&mut self) -> bool {
        todo!()
    }

    fn enable_screensaver(&mut self) -> bool {
        todo!()
    }
}