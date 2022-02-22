package com.github.yoep.popcorn.ui.updater;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformType;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class VersionInfoTest {
    @Test
    void testDownloadForPlatform_whenPlatformIsUnknown_shouldReturnEmpty() {
        var versionInfo = VersionInfo.builder()
                .platforms(Collections.singletonMap("debian.amd64", "https://my-download-link"))
                .build();

        var result = versionInfo.downloadForPlatform(PlatformType.WINDOWS, "x86");

        assertTrue(result.isEmpty(), "Expected no platform download to have been found");
    }

    @Test
    void testDownloadForPlatform_whenPlatformLinkIsKnown_shouldReturnTheDownloadLink() {
        var expectedResult = "https://my-download-link/debian-amd64.deb";
        var versionInfo = VersionInfo.builder()
                .platforms(Collections.singletonMap("debian.amd64", expectedResult))
                .build();

        var result = versionInfo.downloadForPlatform(PlatformType.DEBIAN, "amd64");

        assertTrue(result.isPresent(), "Expected platform download to have been found");
        assertEquals(expectedResult, result.get());
    }
}