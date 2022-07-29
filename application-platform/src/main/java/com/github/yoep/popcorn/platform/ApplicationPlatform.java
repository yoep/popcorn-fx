package com.github.yoep.popcorn.platform;

import com.sun.jna.Library;
import com.sun.jna.Native;

public interface ApplicationPlatform extends Library {
    ApplicationPlatform INSTANCE = Native.load("application_platform", ApplicationPlatform.class);

    PlatformInfo.ByValue platform_info();
}
