package com.github.yoep.popcorn.platform.jna.linux;

import com.sun.jna.platform.unix.X11;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class LinuxUtils {
    public static void disableScreensaver() {
        log.debug("Disabling X11 screensaver");

        try {
            var instance = X11.INSTANCE;
            // TODO: implement X11.DisableScreenSaver
        } catch (UnsatisfiedLinkError ex) {
            log.warn("Failed to disable linux screensaver, {}", ex.getMessage(), ex);
        }
    }
}
