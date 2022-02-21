package com.github.yoep.popcorn.platform.jna.win32;

import com.sun.jna.platform.win32.Kernel32;
import com.sun.jna.platform.win32.WinBase;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class Win32Utils {
    public static void disableScreensaver() {
        log.debug("Disabling Windows screensaver");

        try {
            var instance = Kernel32.INSTANCE;
            instance.SetThreadExecutionState(WinBase.ES_DISPLAY_REQUIRED);
        } catch (UnsatisfiedLinkError ex) {
            log.warn("Failed to disable windows screensaver, {}", ex.getMessage(), ex);
        }
    }
}
