use crate::Platform;

pub struct PlatformLinux {

}

impl PlatformLinux {
    /// Create a new platform instance for linux.
    pub fn new() -> PlatformLinux {
        PlatformLinux{}
    }
}

impl Platform for PlatformLinux {
    fn disable_screensaver(&mut self) -> bool {
        false
    }

    fn enable_screensaver(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod test {

}