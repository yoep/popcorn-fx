package com.github.yoep.popcorn.platform.jna.macos;

import com.sun.jna.platform.mac.CoreFoundation;
import com.sun.jna.ptr.IntByReference;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class MacOsUtils {
    public static void disableScreensaver() {
        log.debug("Disabling MacOS screensaver");

        try {
            var ioKitInstance = ExtendedIOKit.INSTANCE;
            var kIOPMAssertPreventUserIdleSystemSleep = CoreFoundation.CFStringRef.createCFString("PreventUserIdleSystemSleep");
            var reason = CoreFoundation.CFStringRef.createCFString("Media playback application is active");
            var assertionIdRef = new IntByReference(0);

            ioKitInstance.IOPMAssertionCreateWithName(kIOPMAssertPreventUserIdleSystemSleep, 255, reason, assertionIdRef);
        } catch (UnsatisfiedLinkError ex) {
            log.warn("Failed to disable mac screensaver, {}", ex.getMessage(), ex);
        }
    }
}
