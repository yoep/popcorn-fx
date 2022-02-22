package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformInfo;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformType;
import com.github.yoep.popcorn.platform.jna.linux.LinuxUtils;
import com.github.yoep.popcorn.platform.jna.macos.MacOsUtils;
import com.github.yoep.popcorn.platform.jna.win32.Win32Utils;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;

public class PlatformFX implements PlatformProvider {
    @Override
    public boolean isTransparentWindowSupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }

    @Override
    public PlatformInfo platformInfo() {
        return new SimplePlatformInfo(platformType(), com.sun.jna.Platform.ARCH);
    }

    @Override
    public void runOnRenderer(Runnable runnable) {
        Platform.runLater(runnable);
    }

    @Override
    public void disableScreensaver() {
        switch (platformType()) {
            case WINDOWS:
                Win32Utils.disableScreensaver();
                break;
            case MAC:
                MacOsUtils.disableScreensaver();
                break;
            default:
                LinuxUtils.disableScreensaver();
                break;
        }
    }

    private static PlatformType platformType() {
        if (com.sun.jna.Platform.isMac()) {
            return PlatformType.MAC;
        }
        if (com.sun.jna.Platform.isWindows() || com.sun.jna.Platform.isWindowsCE()) {
            return PlatformType.WINDOWS;
        }

        return PlatformType.DEBIAN;
    }
}
