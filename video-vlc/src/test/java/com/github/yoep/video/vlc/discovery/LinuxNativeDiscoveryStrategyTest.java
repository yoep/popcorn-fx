package com.github.yoep.video.vlc.discovery;

import com.sun.jna.Platform;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

class LinuxNativeDiscoveryStrategyTest {
    LinuxNativeDiscoveryStrategy strategy;

    @BeforeEach
    void setUp() {
        strategy = new LinuxNativeDiscoveryStrategy();
    }

    @Test
    void testSupported() {
        assertEquals(Platform.isLinux(), strategy.supported());
    }

    @Test
    void testDiscoveryDirectories() {
        var result = strategy.discoveryDirectories();

        assertFalse(result.isEmpty(), "expected search libraries to have been returned");
    }

    @Test
    void testDiscoveryDirectories_whenVlcPathIsSet_shouldOnlyReturnTheVlcPath() {
        var vlcPath = "/tmp/vlc";
        System.setProperty(LinuxNativeDiscoveryStrategy.VLC_PATH_PROPERTY, vlcPath);

        var result  = strategy.discoveryDirectories();

        assertEquals(1, result.size());
        assertEquals(vlcPath, result.get(0));
    }
}