package com.github.yoep.popcorn.platform.jna.win32;

import com.sun.jna.platform.win32.Kernel32;
import com.sun.jna.platform.win32.WinBase;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.function.Consumer;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class Win32Utils {
    /**
     * Disable the windows screensaver from activating.
     */
    public static void disableScreensaver() {
        log.debug("Disabling Windows screensaver");
        invokeWithKernel(e -> {
            if (e.SetThreadExecutionState(WinBase.ES_CONTINUOUS | WinBase.ES_SYSTEM_REQUIRED | WinBase.ES_DISPLAY_REQUIRED) == 0) {
                log.warn("Failed to disable windows screensaver");
            }
        });
    }

    /**
     * Allow the windows screensaver to be activated.
     */
    public static void allowScreensaver() {
        log.debug("Allowing Windows screensaver");
        invokeWithKernel(e -> {
            if (e.SetThreadExecutionState(WinBase.ES_CONTINUOUS) == 0) {
                log.warn("Failed to allow windows screensaver");
            }
        });
    }

    private static void invokeWithKernel(Consumer<Kernel32> action) {
        Objects.requireNonNull(action, "action cannot be null");
        try {
            var instance = Kernel32.INSTANCE;
            action.accept(instance);
        } catch (UnsatisfiedLinkError ex) {
            log.warn("Failed to invoke kernel32 instance, {}", ex.getMessage(), ex);
        }
    }
}
