package com.github.yoep.video.vlc.discovery;

import com.sun.jna.Platform;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class OsxNativeDiscoveryStrategyTest {
    @Test
    void testSupported() {
        var strategy = new OsxNativeDiscoveryStrategy();

        assertEquals(Platform.isMac(), strategy.supported());
    }
}