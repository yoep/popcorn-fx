package com.github.yoep.popcorn.platform;

public interface NativePlatform {
    PlatformInfo platformInfo();

    void enableScreensaver();

    void disableScreensaver();
}
