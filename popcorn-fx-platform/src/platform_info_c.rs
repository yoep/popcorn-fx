use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::into_c_string;

use crate::popcorn::fx::platform::platform_info::{PlatformInfo, PlatformType};

#[repr(C)]
pub struct PlatformInfoC {
    /// The platform type
    pub platform_type: PlatformType,
    /// The cpu architecture of the platform
    pub arch: *const c_char,
}

impl From<&PlatformInfo> for PlatformInfoC {
    fn from(value: &PlatformInfo) -> Self {
        trace!("Converting platform info to C for {}", value);
        PlatformInfoC {
            platform_type: value.platform_type.clone(),
            arch: into_c_string(value.arch.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::from_c_string;

    use crate::PlatformInfoC;
    use crate::popcorn::fx::platform::platform_info::PlatformInfo;

    #[test]
    fn test_from() {
        let platform = PlatformInfo::new();

        let result = PlatformInfoC::from(&platform);
        let arch_result = from_c_string(result.arch);

        assert_eq!(platform.platform_type, result.platform_type);
        assert_eq!(platform.arch, arch_result);
    }
}