package com.github.yoep.popcorn.platform;

import com.sun.jna.Library;
import com.sun.jna.Native;

public interface ApplicationPlatform extends Library {
    ApplicationPlatform INSTANCE = Native.load("application_platform", ApplicationPlatform.class);

    void init();

    PlatformInfo.ByValue platform_info();

    PlatformC new_platform_c();

    void disable_screensaver(PlatformC platform);

    void enable_screensaver(PlatformC platform);
}
