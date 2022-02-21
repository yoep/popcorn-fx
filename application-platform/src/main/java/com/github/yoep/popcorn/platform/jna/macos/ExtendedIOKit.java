package com.github.yoep.popcorn.platform.jna.macos;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.platform.mac.CoreFoundation;
import com.sun.jna.ptr.IntByReference;

public interface ExtendedIOKit extends Library {
    ExtendedIOKit INSTANCE = Native.load("IOKit", ExtendedIOKit.class);

    int IOPMAssertionCreateWithName(
            CoreFoundation.CFStringRef assertionType,
            int assertionLevel,
            CoreFoundation.CFStringRef assertionName,
            IntByReference assertionID
    );
}
