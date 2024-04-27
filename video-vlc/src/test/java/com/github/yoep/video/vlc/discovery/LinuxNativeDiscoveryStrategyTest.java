package com.github.yoep.video.vlc.discovery;

import com.sun.jna.Platform;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class LinuxNativeDiscoveryStrategyTest {
    @Test
    void testSupported() {
        var strategy = new LinuxNativeDiscoveryStrategy();

        assertEquals(Platform.isLinux(), strategy.supported());
    }
}